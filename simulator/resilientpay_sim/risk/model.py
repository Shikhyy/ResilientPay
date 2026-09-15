from abc import ABC, abstractmethod
from enum import StrEnum

from resilientpay_sim.domain.model import PaymentEnvelope


class RiskClass(StrEnum):
    LOW = "LOW"
    MEDIUM = "MEDIUM"
    HIGH = "HIGH"


class RiskModel(ABC):
    """
    Abstract base class for risk scoring.
    ADR-010 Rule: Risk models are purely advisory and MUST NOT override deterministic authorization.
    """

    @abstractmethod
    def score(self, envelope: PaymentEnvelope) -> float:
        """
        Returns a risk score between 0.0 (safest) and 1.0 (riskiest).
        """
        pass

    def classify(self, envelope: PaymentEnvelope) -> RiskClass:
        """
        Classifies an envelope into a RiskClass based on its score.
        """
        s = self.score(envelope)
        if s < 0.3:
            return RiskClass.LOW
        elif s < 0.7:
            return RiskClass.MEDIUM
        return RiskClass.HIGH


class RuleBasedRiskModel(RiskModel):
    """
    A simple heuristic-based risk model for the prototype simulator.
    """

    def __init__(self, high_amount_minor: int = 100000):
        self.high_amount_minor = high_amount_minor

    def score(self, envelope: PaymentEnvelope) -> float:
        score = 0.0

        # Heuristic 1: Higher amounts are slightly riskier
        amount = envelope.amount.amount_minor
        if amount > self.high_amount_minor:
            score += 0.5
        elif amount > self.high_amount_minor // 2:
            score += 0.2

        # Additional heuristics can be added here (e.g., velocity, new merchant)

        return min(score, 1.0)
