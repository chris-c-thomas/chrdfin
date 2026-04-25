import type { UUID } from "./common.js";

export type CalculatorType =
  | "compound_growth"
  | "retirement"
  | "withdrawal"
  | "options_payoff"
  | "tax_loss"
  | "risk_reward"
  | "position_size"
  | "margin"
  | "dca_vs_lump_sum";

export interface CompoundGrowthInput {
  readonly initial: number;
  readonly contribution: number;
  readonly contributionFrequency: "monthly" | "quarterly" | "annually";
  readonly years: number;
  readonly annualReturn: number;
  readonly inflationRate?: number;
}

export interface CompoundGrowthResult {
  readonly nominalEnd: number;
  readonly realEnd: number;
  readonly totalContributed: number;
  readonly totalGrowth: number;
  readonly schedule: readonly { year: number; balance: number; realBalance: number }[];
}

export interface RetirementInput {
  readonly currentAge: number;
  readonly retirementAge: number;
  readonly currentSavings: number;
  readonly annualContribution: number;
  readonly preRetirementReturn: number;
  readonly postRetirementReturn: number;
  readonly annualWithdrawal: number;
  readonly inflationRate: number;
  readonly endAge: number;
}

export interface RetirementResult {
  readonly accumulationEnd: number;
  readonly depletionAge: number | null;
  readonly successProbability?: number;
  readonly schedule: readonly {
    readonly age: number;
    readonly balance: number;
    readonly contribution: number;
    readonly withdrawal: number;
  }[];
}

export interface OptionsPayoffLeg {
  readonly type: "call" | "put";
  readonly action: "buy" | "sell";
  readonly strike: number;
  readonly premium: number;
  readonly contracts: number;
}

export interface OptionsPayoffInput {
  readonly underlyingPrice: number;
  readonly legs: readonly OptionsPayoffLeg[];
  readonly priceRange?: { min: number; max: number };
}

export interface OptionsPayoffResult {
  readonly maxProfit: number;
  readonly maxLoss: number;
  readonly breakevens: readonly number[];
  readonly payoffCurve: readonly { price: number; pnl: number }[];
}

export interface SavedCalculatorState {
  readonly id: UUID;
  readonly calcType: CalculatorType;
  readonly name: string;
  readonly inputs: Record<string, unknown>;
  readonly results?: Record<string, unknown>;
  readonly createdAt: string;
  readonly updatedAt: string;
}
