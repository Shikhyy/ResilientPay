#!/usr/bin/env python3
"""
E6 — Centralized vs Blockchain Audit Experiment
================================================
Compares the ResilientPay append-only hash-chained ledger (ADR-009) against a
distributed block-based Merkle audit ledger variant (ADR-007).

Evaluated Architectures:
    1. Hash-Chain Local Ledger (ResilientPay Core):
       - Sequential append-only log with SHA-256 cryptographic linkage:
         H_i = SHA-256(H_{i-1} || index || timestamp || tx_id || cbor_payload)
       - O(1) append time, minimal per-tx storage overhead (32-byte hash pointer)
       - O(N) linear full audit verification time

    2. Blockchain / Merkle Audit Variant (Comparative Baseline):
       - Grouped transactions into fixed blocks (batch size B=50)
       - Block header: (block_idx, prev_block_hash, merkle_root, timestamp, validator_sig)
       - Full binary Merkle tree built per block
       - O(B) block finalization overhead + signature verification
       - O(log B) individual inclusion proof, O(N) full ledger verification

Measures:
    - append_throughput_tx_per_sec : write throughput
    - avg_append_latency_us        : latency to record a transaction to audit trail
    - storage_bytes_total          : total bytes occupied on disk/storage
    - storage_bytes_per_tx         : amortized storage overhead per transaction
    - full_audit_latency_ms        : time to cryptographically verify 1,000 transaction trail
    - tamper_detection_latency_us  : latency to detect and locate tampered transaction

Results written to simulator/results/e6_audit_comparison.json.
Reference: docs/07-research-experiments/EXPERIMENT_PLAN_DETAIL.md E6
           docs/09-architecture-decisions/ADR-007-no-blockchain-core.md
           docs/09-architecture-decisions/ADR-009-hash-chain-local-ledger.md
"""

from __future__ import annotations

import hashlib
import json
import math
import random
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional, Tuple

_SIMULATOR_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(_SIMULATOR_ROOT))

import cbor2
from resilientpay_sim.domain.crypto import keypair_from_seed, sign_envelope
from resilientpay_sim.domain.model import (
    ConnectivityState,
    Credential,
    CredentialState,
    Money,
    PaymentEnvelope,
)

SEED = 42
TOTAL_TRANSACTIONS = 1000
BLOCK_SIZE = 50  # For blockchain/Merkle variant
PROTOCOL_VERSION_STR = "1.0"


def _git_commit() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            cwd=str(_SIMULATOR_ROOT),
            stderr=subprocess.DEVNULL,
        ).decode().strip()
    except Exception:
        return "unknown"


# ---------------------------------------------------------------------------
# 1. Hash-Chained Ledger (ADR-009)
# ---------------------------------------------------------------------------
@dataclass
class HashChainEntry:
    index: int
    prev_hash: bytes
    timestamp_unix: int
    tx_id: str
    payload: bytes
    entry_hash: bytes

    def serialized_bytes(self) -> bytes:
        return (
            self.index.to_bytes(8, "big")
            + self.prev_hash
            + self.timestamp_unix.to_bytes(8, "big")
            + self.tx_id.encode("utf-8")
            + len(self.payload).to_bytes(4, "big")
            + self.payload
            + self.entry_hash
        )


class HashChainLedger:
    DOMAIN_TAG = b"RESILIENTPAY-LEDGER-V1:"
    GENESIS_HASH = b"\x00" * 32

    def __init__(self) -> None:
        self.entries: List[HashChainEntry] = []
        self.last_hash: bytes = self.GENESIS_HASH

    def append(self, tx_id: str, timestamp_unix: int, payload: bytes) -> HashChainEntry:
        idx = len(self.entries)
        h = hashlib.sha256()
        h.update(self.DOMAIN_TAG)
        h.update(self.last_hash)
        h.update(idx.to_bytes(8, "big"))
        h.update(timestamp_unix.to_bytes(8, "big"))
        h.update(tx_id.encode("utf-8"))
        h.update(payload)
        entry_hash = h.digest()

        entry = HashChainEntry(
            index=idx,
            prev_hash=self.last_hash,
            timestamp_unix=timestamp_unix,
            tx_id=tx_id,
            payload=payload,
            entry_hash=entry_hash,
        )
        self.entries.append(entry)
        self.last_hash = entry_hash
        return entry

    def verify_integrity(self) -> Tuple[bool, Optional[int]]:
        """Verifies full hash chain. Returns (is_valid, first_invalid_index)."""
        expected_prev = self.GENESIS_HASH
        for idx, entry in enumerate(self.entries):
            if entry.index != idx:
                return False, idx
            if entry.prev_hash != expected_prev:
                return False, idx
            h = hashlib.sha256()
            h.update(self.DOMAIN_TAG)
            h.update(entry.prev_hash)
            h.update(entry.index.to_bytes(8, "big"))
            h.update(entry.timestamp_unix.to_bytes(8, "big"))
            h.update(entry.tx_id.encode("utf-8"))
            h.update(entry.payload)
            computed = h.digest()
            if computed != entry.entry_hash:
                return False, idx
            expected_prev = entry.entry_hash
        return True, None

    def total_bytes(self) -> int:
        return sum(len(e.serialized_bytes()) for e in self.entries)


# ---------------------------------------------------------------------------
# 2. Merkle Tree & Block-Based Audit Ledger (ADR-007 baseline)
# ---------------------------------------------------------------------------
def _merkle_root(leaf_hashes: List[bytes]) -> bytes:
    if not leaf_hashes:
        return b"\x00" * 32
    current = leaf_hashes[:]
    while len(current) > 1:
        next_level = []
        for i in range(0, len(current), 2):
            left = current[i]
            right = current[i + 1] if i + 1 < len(current) else left
            h = hashlib.sha256(b"\x01" + left + right).digest()
            next_level.append(h)
        current = next_level
    return current[0]


@dataclass
class BlockchainBlock:
    block_index: int
    prev_block_hash: bytes
    merkle_root: bytes
    timestamp_unix: int
    validator_sig: bytes
    transactions: List[Tuple[str, int, bytes]]  # tx_id, timestamp, payload
    block_hash: bytes

    def serialized_bytes(self) -> bytes:
        header = (
            self.block_index.to_bytes(8, "big")
            + self.prev_block_hash
            + self.merkle_root
            + self.timestamp_unix.to_bytes(8, "big")
            + self.validator_sig
            + self.block_hash
        )
        body = b"".join(
            tx_id.encode("utf-8") + ts.to_bytes(8, "big") + len(p).to_bytes(4, "big") + p
            for tx_id, ts, p in self.transactions
        )
        return header + body


class BlockchainAuditLedger:
    GENESIS_BLOCK_HASH = b"\x00" * 32

    def __init__(self, validator_sk: bytes, block_size: int = BLOCK_SIZE) -> None:
        self.validator_sk = validator_sk
        self.block_size = block_size
        self.blocks: List[BlockchainBlock] = []
        self.pending_txs: List[Tuple[str, int, bytes]] = []
        self.last_block_hash = self.GENESIS_BLOCK_HASH

    def append(self, tx_id: str, timestamp_unix: int, payload: bytes) -> Optional[BlockchainBlock]:
        self.pending_txs.append((tx_id, timestamp_unix, payload))
        if len(self.pending_txs) >= self.block_size:
            return self._commit_block()
        return None

    def flush(self) -> Optional[BlockchainBlock]:
        if self.pending_txs:
            return self._commit_block()
        return None

    def _commit_block(self) -> BlockchainBlock:
        block_idx = len(self.blocks)
        leaf_hashes = [
            hashlib.sha256(b"\x00" + tx_id.encode("utf-8") + p).digest()
            for tx_id, _, p in self.pending_txs
        ]
        root = _merkle_root(leaf_hashes)
        ts = int(time.time())

        # Simulate PoA validator Ed25519 signature over (block_idx || prev_hash || root)
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
        signer = Ed25519PrivateKey.from_private_bytes(self.validator_sk)
        header_preimage = block_idx.to_bytes(8, "big") + self.last_block_hash + root + ts.to_bytes(8, "big")
        sig = signer.sign(header_preimage)

        block_hash = hashlib.sha256(header_preimage + sig).digest()
        block = BlockchainBlock(
            block_index=block_idx,
            prev_block_hash=self.last_block_hash,
            merkle_root=root,
            timestamp_unix=ts,
            validator_sig=sig,
            transactions=list(self.pending_txs),
            block_hash=block_hash,
        )
        self.blocks.append(block)
        self.last_block_hash = block_hash
        self.pending_txs.clear()
        return block

    def verify_integrity(self) -> Tuple[bool, Optional[int]]:
        """Verifies all block headers, validator signatures, and Merkle tree roots."""
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
        vk = Ed25519PrivateKey.from_private_bytes(self.validator_sk).public_key()
        expected_prev = self.GENESIS_BLOCK_HASH

        for b_idx, block in enumerate(self.blocks):
            if block.block_index != b_idx or block.prev_block_hash != expected_prev:
                return False, b_idx

            # Verify Merkle root of transactions
            leaf_hashes = [
                hashlib.sha256(b"\x00" + tx_id.encode("utf-8") + p).digest()
                for tx_id, _, p in block.transactions
            ]
            computed_root = _merkle_root(leaf_hashes)
            if computed_root != block.merkle_root:
                return False, b_idx

            # Verify validator signature
            header_preimage = (
                block.block_index.to_bytes(8, "big")
                + block.prev_block_hash
                + block.merkle_root
                + block.timestamp_unix.to_bytes(8, "big")
            )
            try:
                vk.verify(block.validator_sig, header_preimage)
            except Exception:
                return False, b_idx

            # Verify block hash
            if hashlib.sha256(header_preimage + block.validator_sig).digest() != block.block_hash:
                return False, b_idx

            expected_prev = block.block_hash

        return True, None

    def total_bytes(self) -> int:
        return sum(len(b.serialized_bytes()) for b in self.blocks)


# ---------------------------------------------------------------------------
# Benchmark Runner
# ---------------------------------------------------------------------------
def main() -> None:
    print("=" * 72)
    print("E6 — Centralized Hash-Chain vs Blockchain Audit Benchmark")
    print("=" * 72)

    rng = random.Random(SEED)
    validator_sk = b"\x77" * 32

    # Synthesize 1,000 Canonical CBOR Payment Envelopes
    print(f"Generating {TOTAL_TRANSACTIONS} canonical payment transactions...")
    privkey, pubkey = keypair_from_seed(b"\x33" * 32)
    merchant_uuid = "00000000-0000-0000-0000-000000000004"
    cred_uuid = "00000000-0000-0000-0000-000000000002"

    test_txs: List[Tuple[str, int, bytes]] = []
    base_time = 1700000000
    for i in range(1, TOTAL_TRANSACTIONS + 1):
        tx_time = base_time + (i * 60)
        env = PaymentEnvelope(
            protocol_version=1,
            tx_id=f"00000000-0000-0000-0006-{i:012x}",
            credential_id=cred_uuid,
            payer_key_id=cred_uuid,
            merchant_id=merchant_uuid,
            amount=Money(rng.randint(100, 50000), "INR"),
            counter=i,
            nonce=rng.randbytes(16),
            created_at_unix=tx_time,
            expires_at_unix=tx_time + 3600,
        )
        sig = sign_envelope(env, privkey)
        # Canonical CBOR serialization of envelope + signature
        payload = cbor2.dumps({
            "env": {
                "v": env.protocol_version,
                "id": env.tx_id,
                "cid": env.credential_id,
                "pkid": env.payer_key_id,
                "mid": env.merchant_id,
                "amt": env.amount.amount_minor,
                "cur": env.amount.currency,
                "ctr": env.counter,
                "nce": env.nonce,
                "iat": env.created_at_unix,
                "exp": env.expires_at_unix,
            },
            "sig": sig,
        })
        test_txs.append((env.tx_id, tx_time, payload))

    # -----------------------------------------------------------------------
    # Benchmark 1: Centralized Hash-Chained Ledger
    # -----------------------------------------------------------------------
    print("\n[1/2] Benchmarking Centralized Hash-Chain Ledger (ADR-009)...")
    hash_chain = HashChainLedger()
    hc_latencies_us = []

    t_start = time.perf_counter()
    for tx_id, ts, payload in test_txs:
        t0 = time.perf_counter()
        hash_chain.append(tx_id, ts, payload)
        t1 = time.perf_counter()
        hc_latencies_us.append((t1 - t0) * 1e6)
    t_end = time.perf_counter()

    hc_total_time = t_end - t_start
    hc_throughput = TOTAL_TRANSACTIONS / hc_total_time
    hc_avg_latency_us = sum(hc_latencies_us) / len(hc_latencies_us)
    hc_sorted = sorted(hc_latencies_us)
    hc_p95_latency_us = hc_sorted[int(0.95 * len(hc_sorted))]
    hc_storage_bytes = hash_chain.total_bytes()
    hc_storage_per_tx = hc_storage_bytes / TOTAL_TRANSACTIONS

    # Verify integrity
    t0 = time.perf_counter()
    valid_hc, err_idx = hash_chain.verify_integrity()
    t1 = time.perf_counter()
    hc_audit_latency_ms = (t1 - t0) * 1e3
    assert valid_hc and err_idx is None, "Hash-chain integrity verification failed!"

    # Tamper detection benchmark: alter record #500
    hash_chain.entries[500].payload = b"TAMPERED_PAYLOAD"
    t0 = time.perf_counter()
    tamper_valid, tamper_idx = hash_chain.verify_integrity()
    t1 = time.perf_counter()
    hc_tamper_detection_us = (t1 - t0) * 1e6
    assert not tamper_valid and tamper_idx == 500, "Hash-chain failed to detect tamper!"
    # Restore record
    hash_chain.entries[500].payload = test_txs[500][2]

    # -----------------------------------------------------------------------
    # Benchmark 2: Blockchain / Merkle Audit Ledger
    # -----------------------------------------------------------------------
    print("\n[2/2] Benchmarking Blockchain Merkle Ledger (ADR-007)...")
    bc_ledger = BlockchainAuditLedger(validator_sk=validator_sk, block_size=BLOCK_SIZE)
    bc_latencies_us = []

    t_start = time.perf_counter()
    for tx_id, ts, payload in test_txs:
        t0 = time.perf_counter()
        bc_ledger.append(tx_id, ts, payload)
        t1 = time.perf_counter()
        bc_latencies_us.append((t1 - t0) * 1e6)
    bc_ledger.flush()
    t_end = time.perf_counter()

    bc_total_time = t_end - t_start
    bc_throughput = TOTAL_TRANSACTIONS / bc_total_time
    bc_avg_latency_us = sum(bc_latencies_us) / len(bc_latencies_us)
    bc_sorted = sorted(bc_latencies_us)
    bc_p95_latency_us = bc_sorted[int(0.95 * len(bc_sorted))]
    bc_storage_bytes = bc_ledger.total_bytes()
    bc_storage_per_tx = bc_storage_bytes / TOTAL_TRANSACTIONS

    # Verify integrity
    t0 = time.perf_counter()
    valid_bc, err_b_idx = bc_ledger.verify_integrity()
    t1 = time.perf_counter()
    bc_audit_latency_ms = (t1 - t0) * 1e3
    assert valid_bc and err_b_idx is None, "Blockchain integrity verification failed!"

    # Tamper detection benchmark: alter record in block #10
    bc_ledger.blocks[10].transactions[0] = (
        bc_ledger.blocks[10].transactions[0][0],
        bc_ledger.blocks[10].transactions[0][1],
        b"TAMPERED_TRANSACTION_PAYLOAD",
    )
    t0 = time.perf_counter()
    tamper_bc_valid, tamper_b_idx = bc_ledger.verify_integrity()
    t1 = time.perf_counter()
    bc_tamper_detection_us = (t1 - t0) * 1e6
    assert not tamper_bc_valid and tamper_b_idx == 10, "Blockchain failed to detect tamper!"
    # Restore block
    bc_ledger.blocks[10].transactions[0] = (
        bc_ledger.blocks[10].transactions[0][0],
        bc_ledger.blocks[10].transactions[0][1],
        test_txs[10 * BLOCK_SIZE][2],
    )

    # -----------------------------------------------------------------------
    # Report Results
    # -----------------------------------------------------------------------
    print("\n" + "=" * 72)
    print(f"{'Metric':<36} | {'Hash-Chain (ResilientPay)':<18} | {'Blockchain / Merkle':<18}")
    print("-" * 72)
    print(f"{'Throughput (tx/s)':<36} | {hc_throughput:>18.1f} | {bc_throughput:>18.1f}")
    print(f"{'Avg Append Latency (µs)':<36} | {hc_avg_latency_us:>18.2f} | {bc_avg_latency_us:>18.2f}")
    print(f"{'P95 Append Latency (µs)':<36} | {hc_p95_latency_us:>18.2f} | {bc_p95_latency_us:>18.2f}")
    print(f"{'Storage Total (Bytes)':<36} | {hc_storage_bytes:>18,d} | {bc_storage_bytes:>18,d}")
    print(f"{'Storage Overhead (Bytes/tx)':<36} | {hc_storage_per_tx:>18.1f} | {bc_storage_per_tx:>18.1f}")
    print(f"{'Audit Verify Latency (1k tx) (ms)':<36} | {hc_audit_latency_ms:>18.2f} | {bc_audit_latency_ms:>18.2f}")
    print(f"{'Tamper Detection Time (µs)':<36} | {hc_tamper_detection_us:>18.2f} | {bc_tamper_detection_us:>18.2f}")
    print("=" * 72)

    # Output JSON artifact
    out_file = _SIMULATOR_ROOT / "results" / "e6_audit_comparison.json"
    payload_json = {
        "experiment": "E6_audit_comparison",
        "schema_version": 1,
        "metadata": {
            "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "git_commit": _git_commit(),
            "seed": SEED,
            "total_transactions": TOTAL_TRANSACTIONS,
            "block_size": BLOCK_SIZE,
            "protocol_version": PROTOCOL_VERSION_STR,
        },
        "results": {
            "hash_chain_ledger": {
                "throughput_tx_per_sec": round(hc_throughput, 1),
                "avg_append_latency_us": round(hc_avg_latency_us, 2),
                "p95_append_latency_us": round(hc_p95_latency_us, 2),
                "storage_bytes_total": hc_storage_bytes,
                "storage_bytes_per_tx": round(hc_storage_per_tx, 1),
                "full_audit_latency_ms": round(hc_audit_latency_ms, 2),
                "tamper_detection_latency_us": round(hc_tamper_detection_us, 2),
            },
            "blockchain_merkle_ledger": {
                "throughput_tx_per_sec": round(bc_throughput, 1),
                "avg_append_latency_us": round(bc_avg_latency_us, 2),
                "p95_append_latency_us": round(bc_p95_latency_us, 2),
                "storage_bytes_total": bc_storage_bytes,
                "storage_bytes_per_tx": round(bc_storage_per_tx, 1),
                "full_audit_latency_ms": round(bc_audit_latency_ms, 2),
                "tamper_detection_latency_us": round(bc_tamper_detection_us, 2),
            },
            "ratio_hash_chain_vs_blockchain": {
                "throughput_advantage_factor": round(hc_throughput / bc_throughput, 2),
                "storage_saving_percent": round((1 - hc_storage_bytes / bc_storage_bytes) * 100, 2),
                "audit_verification_speedup_factor": round(bc_audit_latency_ms / hc_audit_latency_ms, 2),
            },
        },
    }
    with open(out_file, "w") as f:
        json.dump(payload_json, f, indent=2)

    print(f"\nResults written to: {out_file}")


if __name__ == "__main__":
    main()
