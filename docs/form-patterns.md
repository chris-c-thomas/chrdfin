# Form Patterns — chrdfin

## Purpose

Conventions for every form, calculator, filter bar, and config panel in chrdfin. Covers the React Hook Form + Zod stack, field component primitives, search-param sync, async validation, field arrays, multi-section forms, and submit/mutation integration.

This is the implementation contract for any UI that accepts structured user input. If a component takes more than two coordinated values from the user, it follows these patterns.

**Companion documents:**

- `docs/ui-design-system.md` — typography, density, control heights
- `docs/ui-component-recipes.md` — `<DeltaValue>`, formatters, `cn()`, `useTauriCommand`
- `docs/route-conventions.md` — search param schemas (some forms sync to URL)
- `docs/data-fetching-patterns.md` — mutation patterns for form submission
- `docs/type-definitions-reference.md` — shared Zod schemas in `@chrdfin/types`

---

## Package Boundaries

| Concern | Location |
|---|---|
| Field primitives | `packages/@chrdfin/ui/src/components/form/*.tsx` |
| `<Form>` + `<FormField>` wiring | `packages/@chrdfin/ui/src/components/form/form.tsx` |
| Form-specific hooks (e.g. `useFormSearchSync`) | `packages/@chrdfin/ui/src/hooks/use-form-search-sync.ts` |
| Domain-specific form hooks | Colocated with route, e.g. `apps/desktop/src/routes/tools/calculators/hooks/use-compound-form.ts` |
| Input Zod schemas (form values) | `packages/@chrdfin/types/src/forms.ts` |

---

## Stack

| Package | Use |
|---|---|
| `react-hook-form` | Form state, validation orchestration, performance (uncontrolled inputs by default) |
| `@hookform/resolvers/zod` | Bridge Zod schemas → RHF validation |
| `zod` | Schema definition (single source of truth) |

**Why this stack:**

- React Hook Form is uncontrolled-by-default, which means typing in an input doesn't re-render the entire form. For chrdfin's dense forms (backtest config can have 30+ fields), this matters.
- Zod schemas defined once serve as: form validators, search param parsers (per `route-conventions.md`), Tauri command input validators, and TypeScript types.
- The `zodResolver` adapter is officially maintained, well-tested, and small.

---

## 1. Schema-First Forms

The Zod schema is the source of truth. The TypeScript type is derived; the validator is derived; the field definitions reference field names from the schema.

### File: `packages/@chrdfin/types/src/forms.ts`

```typescript
import { z } from "zod";

/* ============================================================
   Calculator forms
   ============================================================ */

export const CompoundInterestSchema = z.object({
  initialAmount: z
    .number({ invalid_type_error: "Required" })
    .positive("Must be positive"),
  monthlyContribution: z
    .number({ invalid_type_error: "Required" })
    .nonnegative(),
  annualRate: z
    .number({ invalid_type_error: "Required" })
    .min(-50, "Min -50%")
    .max(50, "Max 50%"),
  years: z
    .number({ invalid_type_error: "Required" })
    .int("Whole years only")
    .min(1)
    .max(100),
  compoundFrequency: z
    .enum(["monthly", "quarterly", "annually", "daily"])
    .default("monthly"),
});

export type CompoundInterestInput = z.infer<typeof CompoundInterestSchema>;

/* ============================================================
   Transaction entry
   ============================================================ */

export const TransactionInputSchema = z.object({
  portfolioId: z.string().uuid(),
  symbol: z
    .string()
    .min(1, "Required")
    .max(10)
    .regex(/^[A-Z0-9.\-]+$/, "Invalid format")
    .transform((s) => s.toUpperCase()),
  type: z.enum(["buy", "sell", "dividend", "split", "transfer"]),
  date: z
    .string()
    .regex(/^\d{4}-\d{2}-\d{2}$/, "Use YYYY-MM-DD"),
  shares: z
    .number({ invalid_type_error: "Required" })
    .positive("Must be positive"),
  price: z
    .number({ invalid_type_error: "Required" })
    .nonnegative(),
  fees: z.number().nonnegative().default(0),
  notes: z.string().max(500).default(""),
});

export type TransactionInput = z.infer<typeof TransactionInputSchema>;

/* ============================================================
   Backtest configuration
   ============================================================ */

export const BacktestHoldingSchema = z.object({
  symbol: z
    .string()
    .min(1)
    .max(10)
    .regex(/^[A-Z0-9.\-]+$/),
  weight: z.number().min(0).max(1),
});

export const BacktestConfigSchema = z
  .object({
    holdings: z
      .array(BacktestHoldingSchema)
      .min(1, "At least one holding required")
      .max(50),
    start: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    end: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    initial: z.number().positive().default(10_000),
    rebalance: z
      .enum(["none", "monthly", "quarterly", "annually"])
      .default("annually"),
    benchmark: z
      .string()
      .regex(/^[A-Z0-9.\-]+$/)
      .default("SPY"),
  })
  .superRefine((value, ctx) => {
    // Cross-field: weights must sum to 1.0 (within tolerance)
    const total = value.holdings.reduce((sum, h) => sum + h.weight, 0);
    if (Math.abs(total - 1) > 0.001) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: `Weights must sum to 1.0 (currently ${total.toFixed(3)})`,
        path: ["holdings"],
      });
    }
    // Cross-field: end > start
    if (new Date(value.end) <= new Date(value.start)) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: "End must be after start",
        path: ["end"],
      });
    }
  });

export type BacktestConfig = z.infer<typeof BacktestConfigSchema>;
```

**Tradeoffs:**

- Custom error messages on every constraint (`"Required"`, `"Must be positive"`). Default Zod messages reference internal codes (`"Number must be greater than 0"`) that aren't great UX. Customize once at schema definition, reuse everywhere.
- `superRefine` for cross-field validation (sum-to-one, date ordering). The error is attached to a specific path so the UI can show it next to the relevant control.
- `transform()` for normalization (uppercase tickers). The transformed value is what the form's submit handler receives — the user can type lowercase and the schema fixes it.
- Default values via `.default(...)` mean `parse({})` produces a complete object. RHF picks these up as initial values when no overrides are passed.

---

## 2. The Form Foundation

A thin wrapper around RHF that wires Zod resolution, default values, and a field context for child components. Modeled on shadcn/ui's `<Form>` primitive but trimmed to chrdfin's density.

### File: `packages/@chrdfin/ui/src/components/form/form.tsx`

```typescript
import {
  FormProvider,
  useForm,
  useFormContext,
  useFormState,
  Controller,
  type ControllerProps,
  type FieldPath,
  type FieldValues,
  type SubmitHandler,
  type UseFormProps,
  type UseFormReturn,
} from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  createContext,
  useContext,
  useId,
  type ReactNode,
} from "react";
import type { z } from "zod";
import { cn } from "../../lib/utils";

/* ============================================================
   Form root
   ============================================================ */

export interface FormProps<TSchema extends z.ZodType<FieldValues, z.ZodTypeDef, unknown>>
  extends Omit<UseFormProps<z.infer<TSchema>>, "resolver"> {
  schema: TSchema;
  onSubmit: SubmitHandler<z.infer<TSchema>>;
  children: ReactNode;
  className?: string;
  /** Disable native HTML5 form validation (we use Zod). Default true. */
  noValidate?: boolean;
}

/**
 * Schema-driven form wrapper. Extracts the resolver from a Zod schema
 * and exposes RHF state via context.
 *
 * @example
 *   <Form
 *     schema={CompoundInterestSchema}
 *     defaultValues={{ initialAmount: 10000, ...rest }}
 *     onSubmit={(values) => handleCalculate(values)}
 *   >
 *     <FormField name="initialAmount" label="Initial">
 *       <NumberField />
 *     </FormField>
 *     <Button type="submit">Calculate</Button>
 *   </Form>
 */
export function Form<TSchema extends z.ZodType<FieldValues, z.ZodTypeDef, unknown>>({
  schema,
  onSubmit,
  defaultValues,
  children,
  className,
  noValidate = true,
  mode = "onBlur",
  ...rest
}: FormProps<TSchema>): JSX.Element {
  const methods = useForm<z.infer<TSchema>>({
    resolver: zodResolver(schema),
    defaultValues,
    mode,
    ...rest,
  });

  return (
    <FormProvider {...methods}>
      <form
        onSubmit={methods.handleSubmit(onSubmit)}
        className={cn("flex flex-col gap-4", className)}
        noValidate={noValidate}
      >
        {children}
      </form>
    </FormProvider>
  );
}

/* ============================================================
   Field context (label + error wiring)
   ============================================================ */

interface FormFieldContextValue {
  name: string;
  id: string;
  errorId: string;
  hasError: boolean;
}

const FormFieldContext = createContext<FormFieldContextValue | null>(null);

export function useFormField(): FormFieldContextValue {
  const ctx = useContext(FormFieldContext);
  if (!ctx) throw new Error("useFormField must be used inside <FormField>");
  return ctx;
}

/* ============================================================
   FormField — labeled, error-aware wrapper
   ============================================================ */

export interface FormFieldProps<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
> extends Omit<ControllerProps<TFieldValues, TName>, "render"> {
  label?: ReactNode;
  helperText?: ReactNode;
  /** Render the label inline (left of input) instead of stacked. */
  inline?: boolean;
  className?: string;
  children: ReactNode;
}

/**
 * Wires a controlled child input to RHF state with label + error
 * presentation. Children read RHF state via `useController` plus the
 * field context (id, errorId).
 *
 * @example
 *   <FormField name="amount" label="Amount">
 *     <NumberField />
 *   </FormField>
 */
export function FormField<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
>({
  name,
  label,
  helperText,
  inline = false,
  className,
  children,
  ...controllerProps
}: FormFieldProps<TFieldValues, TName>): JSX.Element {
  const { control } = useFormContext<TFieldValues>();
  const { errors } = useFormState({ control, name });
  const id = useId();
  const errorId = `${id}-error`;
  const fieldError = errors[name as string];

  return (
    <FormFieldContext.Provider
      value={{
        name: name as string,
        id,
        errorId,
        hasError: !!fieldError,
      }}
    >
      <div
        className={cn(
          inline ? "grid grid-cols-[140px_1fr] items-center gap-3" : "flex flex-col gap-1",
          className,
        )}
      >
        {label ? (
          <label
            htmlFor={id}
            className="text-xs text-muted-foreground"
          >
            {label}
          </label>
        ) : null}
        <Controller name={name} control={control} {...controllerProps} render={() => <>{children}</>} />
        {fieldError ? (
          <p id={errorId} className="text-xs text-destructive">
            {(fieldError.message as string) ?? "Invalid"}
          </p>
        ) : helperText ? (
          <p className="text-xs text-muted-foreground">{helperText}</p>
        ) : null}
      </div>
    </FormFieldContext.Provider>
  );
}
```

**Tradeoffs:**

- `mode: "onBlur"` is the default — validation on blur, not on every keystroke. Less visual noise during typing; errors appear when the user moves to the next field. Override per-form to `onChange` for filter UIs where instant feedback matters.
- `<Controller>` wraps every field. Alternative is RHF's `register()` API which is faster (less wrapping) but requires the field to forward refs and accept native input props. Controller is more flexible and the perf cost is invisible at chrdfin's form complexity.
- The `<FormFieldContext>` exposes `id` and `errorId` so child inputs can wire up `aria-describedby` for accessibility. Required for keyboard-only users.
- Label is left-aligned and 140px wide in inline mode. This matches the form-density target — long labels truncate but data alignment is preserved.

---

## 3. Field Primitives

Atomic input components consumed via `<FormField>`. Each reads `useFormContext()` and `useFormField()` for wiring.

### File: `packages/@chrdfin/ui/src/components/form/text-field.tsx`

```typescript
import { useController, useFormContext } from "react-hook-form";
import { useFormField } from "./form";
import { cn } from "../../lib/utils";
import type { InputHTMLAttributes } from "react";

export interface TextFieldProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "name" | "value" | "onChange"> {
  /** Compact (h-7, 28px) or default (h-8, 32px). Default "default". */
  density?: "default" | "compact";
}

export function TextField({
  density = "default",
  className,
  type = "text",
  ...rest
}: TextFieldProps): JSX.Element {
  const { control } = useFormContext();
  const { name, id, errorId, hasError } = useFormField();
  const { field } = useController({ name, control });

  return (
    <input
      {...rest}
      {...field}
      id={id}
      type={type}
      aria-describedby={hasError ? errorId : undefined}
      aria-invalid={hasError || undefined}
      className={cn(
        "border bg-background px-2 text-sm text-foreground",
        "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
        density === "compact" ? "h-7" : "h-8",
        hasError ? "border-destructive" : "border-input",
        className,
      )}
    />
  );
}
```

### File: `packages/@chrdfin/ui/src/components/form/number-field.tsx`

```typescript
import { useController, useFormContext } from "react-hook-form";
import { useFormField } from "./form";
import { cn } from "../../lib/utils";
import type { InputHTMLAttributes } from "react";

export interface NumberFieldProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "name" | "value" | "onChange" | "type"> {
  density?: "default" | "compact";
  /** Optional unit label (e.g. "%", "$"). Rendered as a non-interactive suffix. */
  suffix?: string;
  /** Optional unit label rendered before the input (e.g. "$"). */
  prefix?: string;
}

/**
 * Number input with proper coercion to number type.
 *
 * RHF stores empty inputs as `""` by default — we convert empty to
 * `undefined` so Zod's optional/required logic fires correctly. Numeric
 * coercion happens on blur; during typing the raw string is preserved
 * to allow partial input ("1.").
 */
export function NumberField({
  density = "default",
  suffix,
  prefix,
  className,
  step = "any",
  ...rest
}: NumberFieldProps): JSX.Element {
  const { control } = useFormContext();
  const { name, id, errorId, hasError } = useFormField();
  const { field } = useController({ name, control });

  // RHF stores the value; we coerce when a complete number is parsed.
  const stringValue =
    field.value === undefined || field.value === null
      ? ""
      : String(field.value);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value;
    if (raw === "") {
      field.onChange(undefined);
      return;
    }
    // Allow partial/intermediate strings during typing
    const parsed = Number(raw);
    if (!Number.isNaN(parsed)) {
      field.onChange(parsed);
    } else {
      // Pass through for the user to keep typing (e.g. "-")
      field.onChange(raw);
    }
  };

  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    // On blur, coerce or set undefined
    const raw = e.target.value;
    if (raw === "") {
      field.onChange(undefined);
    } else {
      const parsed = Number(raw);
      if (Number.isNaN(parsed)) {
        field.onChange(undefined);
      } else {
        field.onChange(parsed);
      }
    }
    field.onBlur();
  };

  const heightClass = density === "compact" ? "h-7" : "h-8";

  return (
    <div
      className={cn(
        "flex items-stretch border bg-background",
        hasError ? "border-destructive" : "border-input",
        "focus-within:ring-1 focus-within:ring-ring",
        heightClass,
        className,
      )}
    >
      {prefix ? (
        <span className="flex items-center px-2 text-xs text-muted-foreground border-r border-input bg-muted/50">
          {prefix}
        </span>
      ) : null}
      <input
        {...rest}
        id={id}
        type="text"
        inputMode="decimal"
        step={step}
        value={stringValue}
        onChange={handleChange}
        onBlur={handleBlur}
        aria-describedby={hasError ? errorId : undefined}
        aria-invalid={hasError || undefined}
        className={cn(
          "flex-1 border-0 bg-transparent px-2 text-sm text-foreground font-mono tabular-nums",
          "focus:outline-none",
          "[appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none",
        )}
      />
      {suffix ? (
        <span className="flex items-center px-2 text-xs text-muted-foreground border-l border-input bg-muted/50">
          {suffix}
        </span>
      ) : null}
    </div>
  );
}
```

### File: `packages/@chrdfin/ui/src/components/form/select-field.tsx`

```typescript
import { useController, useFormContext } from "react-hook-form";
import { useFormField } from "./form";
import { cn } from "../../lib/utils";

export interface SelectFieldOption<T extends string> {
  value: T;
  label: string;
}

export interface SelectFieldProps<T extends string> {
  options: ReadonlyArray<SelectFieldOption<T>>;
  density?: "default" | "compact";
  className?: string;
}

/**
 * Native select for compact, dense usage. For richer UX (search,
 * multi-select), use a Combobox — defined separately.
 */
export function SelectField<T extends string>({
  options,
  density = "default",
  className,
}: SelectFieldProps<T>): JSX.Element {
  const { control } = useFormContext();
  const { name, id, errorId, hasError } = useFormField();
  const { field } = useController({ name, control });

  return (
    <select
      {...field}
      id={id}
      aria-describedby={hasError ? errorId : undefined}
      aria-invalid={hasError || undefined}
      className={cn(
        "border bg-background px-2 text-sm text-foreground",
        "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
        density === "compact" ? "h-7" : "h-8",
        hasError ? "border-destructive" : "border-input",
        className,
      )}
    >
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}
```

### File: `packages/@chrdfin/ui/src/components/form/date-field.tsx`

```typescript
import { useController, useFormContext } from "react-hook-form";
import { useFormField } from "./form";
import { cn } from "../../lib/utils";

export interface DateFieldProps {
  density?: "default" | "compact";
  /** Min date in YYYY-MM-DD */
  min?: string;
  /** Max date in YYYY-MM-DD */
  max?: string;
  className?: string;
}

/**
 * Native date input. Stores ISO YYYY-MM-DD strings to match Zod schema
 * conventions and Rust serde::Date deserialization.
 */
export function DateField({
  density = "default",
  min,
  max,
  className,
}: DateFieldProps): JSX.Element {
  const { control } = useFormContext();
  const { name, id, errorId, hasError } = useFormField();
  const { field } = useController({ name, control });

  return (
    <input
      {...field}
      id={id}
      type="date"
      min={min}
      max={max}
      aria-describedby={hasError ? errorId : undefined}
      aria-invalid={hasError || undefined}
      className={cn(
        "border bg-background px-2 text-sm text-foreground font-mono tabular-nums",
        "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
        density === "compact" ? "h-7" : "h-8",
        hasError ? "border-destructive" : "border-input",
        className,
      )}
    />
  );
}
```

**Tradeoffs:**

- Native `<input type="date">` is used rather than a JS date picker. Pros: no library, consistent OS UX, keyboard-native. Cons: limited styling (browsers render their own popup). For chrdfin's terminal aesthetic the native control is fine; if visual consistency across views ever matters more, swap in a custom picker per-form.
- Native `<select>` for the same reasons. The Combobox pattern (with shadcn `<Command>`) is reserved for ticker autocomplete and other large option sets.
- `NumberField` accepts intermediate strings during typing ("-", "1.") to avoid jumpy revert-to-undefined behavior. Coercion happens on blur. The schema gets a number or undefined.
- Density variants (`h-7` vs `h-8`) match the table density variants in `<DataTable>`.

---

## 4. Reference Recipe: Compound Interest Calculator

The simplest end-to-end form. Phase 6 deliverable; included here as the canonical small-form pattern.

### File: `apps/desktop/src/routes/tools/calculators/compound-interest.lazy.tsx`

```typescript
import { createLazyFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import {
  Form,
  FormField,
  NumberField,
  SelectField,
} from "@chrdfin/ui/components/form";
import { Button } from "@chrdfin/ui/components/button";
import {
  CompoundInterestSchema,
  type CompoundInterestInput,
} from "@chrdfin/types";
import { formatCurrency } from "@chrdfin/ui/lib/format";

export const Route = createLazyFileRoute("/tools/calculators/compound-interest")({
  component: CompoundInterestPage,
});

interface CompoundResult {
  finalAmount: number;
  totalContributions: number;
  totalInterest: number;
}

function compute(input: CompoundInterestInput): CompoundResult {
  const periodsPerYear =
    input.compoundFrequency === "monthly" ? 12 :
    input.compoundFrequency === "quarterly" ? 4 :
    input.compoundFrequency === "daily" ? 365 :
    1;
  const r = input.annualRate / 100 / periodsPerYear;
  const n = input.years * periodsPerYear;
  const fvLump = input.initialAmount * Math.pow(1 + r, n);
  const fvSeries =
    input.monthlyContribution > 0
      ? input.monthlyContribution * 12 / periodsPerYear * (Math.pow(1 + r, n) - 1) / r
      : 0;
  const finalAmount = fvLump + fvSeries;
  const totalContributions = input.initialAmount + input.monthlyContribution * 12 * input.years;
  return {
    finalAmount,
    totalContributions,
    totalInterest: finalAmount - totalContributions,
  };
}

const FREQUENCY_OPTIONS = [
  { value: "monthly", label: "Monthly" },
  { value: "quarterly", label: "Quarterly" },
  { value: "annually", label: "Annually" },
  { value: "daily", label: "Daily" },
] as const;

function CompoundInterestPage(): JSX.Element {
  const [result, setResult] = useState<CompoundResult | null>(null);

  return (
    <div className="grid grid-cols-[400px_1fr] gap-6 p-6">
      <Form
        schema={CompoundInterestSchema}
        defaultValues={{
          initialAmount: 10_000,
          monthlyContribution: 500,
          annualRate: 7,
          years: 30,
          compoundFrequency: "monthly",
        }}
        onSubmit={(values) => setResult(compute(values))}
      >
        <FormField name="initialAmount" label="Initial Amount" inline>
          <NumberField prefix="$" />
        </FormField>
        <FormField name="monthlyContribution" label="Monthly Contribution" inline>
          <NumberField prefix="$" />
        </FormField>
        <FormField name="annualRate" label="Annual Return" inline>
          <NumberField suffix="%" />
        </FormField>
        <FormField name="years" label="Years" inline>
          <NumberField />
        </FormField>
        <FormField name="compoundFrequency" label="Compounding" inline>
          <SelectField options={FREQUENCY_OPTIONS} />
        </FormField>
        <div className="pt-2">
          <Button type="submit">Calculate</Button>
        </div>
      </Form>

      {result ? <ResultPanel result={result} /> : null}
    </div>
  );
}

function ResultPanel({ result }: { result: CompoundResult }): JSX.Element {
  return (
    <div className="flex flex-col gap-4">
      <div className="text-xs text-muted-foreground uppercase">Final Amount</div>
      <div className="text-xl font-mono tabular-nums">
        {formatCurrency(result.finalAmount, { precision: 0 })}
      </div>
      <div className="grid grid-cols-2 gap-6 pt-4">
        <div>
          <div className="text-xs text-muted-foreground uppercase">Contributions</div>
          <div className="text-md font-mono tabular-nums">
            {formatCurrency(result.totalContributions, { precision: 0 })}
          </div>
        </div>
        <div>
          <div className="text-xs text-muted-foreground uppercase">Interest Earned</div>
          <div className="text-md font-mono tabular-nums">
            {formatCurrency(result.totalInterest, { precision: 0 })}
          </div>
        </div>
      </div>
    </div>
  );
}
```

**Tradeoffs:**

- Computation runs in TypeScript on submit, not via Tauri command. Calculators are simple closed-form — no need for Rust round-trip. The Rust backend will, however, be the path for backtest and Monte Carlo because they involve historical data.
- Submit handler captures result in local state. For "save to history" later, swap this for a `useTauriMutation` that writes to the `saved_calculator_states` table.

---

## 5. Search-Param-Synced Forms

For shareable configurations (backtest, screener), the form is bidirectionally bound to the URL search params. URL is the source of truth; the form is a controlled view of it.

### Pattern

1. The route declares a search param schema (see `route-conventions.md` section 4).
2. The form's `defaultValues` come from the validated search.
3. On valid submit, the form pushes the new values to the URL via `useNavigate`.
4. When the URL changes (back/forward, deep link, command palette navigation), the form re-renders.

### File: `packages/@chrdfin/ui/src/hooks/use-form-search-sync.ts`

```typescript
import { useEffect } from "react";
import type { FieldValues, UseFormReturn } from "react-hook-form";

/**
 * Synchronize an RHF form with TanStack Router search params.
 *
 * One-way binding: when search params change (back/forward, external
 * navigation), reset the form to match. The form-side push back to URL
 * is the consumer's responsibility (typically in onSubmit).
 *
 * @param form — RHF methods returned by useForm
 * @param search — current search params object from useSearch()
 */
export function useFormSearchSync<TValues extends FieldValues>(
  form: UseFormReturn<TValues>,
  search: TValues,
): void {
  useEffect(() => {
    // reset() re-applies defaultValues, marking the form as pristine.
    // Use keepDirtyValues to avoid clobbering in-progress edits when
    // search arrives mid-typing (rare but possible).
    form.reset(search, { keepDirtyValues: true });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [JSON.stringify(search)]);
}
```

### Reference: Backtest config form (Phase 2)

```typescript
// apps/desktop/src/routes/analysis/backtest.lazy.tsx (Phase 2)
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import {
  Form,
  FormField,
  NumberField,
  SelectField,
  DateField,
  useFormSearchSync,
} from "@chrdfin/ui/components/form";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { BacktestConfigSchema, type BacktestConfig } from "@chrdfin/types";
import { HoldingsArray } from "./components/holdings-array";

export const Route = createLazyFileRoute("/analysis/backtest")({
  component: BacktestPage,
});

function BacktestPage(): JSX.Element {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: "/analysis/backtest" });

  // Build initial values from search params, falling back to defaults.
  const initialValues: BacktestConfig = {
    holdings:
      search.tickers && search.tickers.length > 0
        ? search.tickers.map((symbol, i) => ({
            symbol,
            weight: search.weights?.[i] ?? 1 / (search.tickers?.length ?? 1),
          }))
        : [
            { symbol: "SPY", weight: 0.6 },
            { symbol: "AGG", weight: 0.4 },
          ],
    start: search.start ?? "2005-01-01",
    end: search.end ?? new Date().toISOString().slice(0, 10),
    initial: search.initial ?? 10_000,
    rebalance: search.rebalance ?? "annually",
    benchmark: "SPY",
  };

  const form = useForm<BacktestConfig>({
    resolver: zodResolver(BacktestConfigSchema),
    defaultValues: initialValues,
    mode: "onBlur",
  });

  // Re-hydrate when URL changes (back/forward, command palette).
  useFormSearchSync(form, initialValues as BacktestConfig);

  const handleSubmit = (values: BacktestConfig) => {
    navigate({
      search: {
        tickers: values.holdings.map((h) => h.symbol),
        weights: values.holdings.map((h) => h.weight),
        start: values.start,
        end: values.end,
        initial: values.initial,
        rebalance: values.rebalance,
      },
    });
  };

  return (
    <FormProvider {...form}>
      <form
        onSubmit={form.handleSubmit(handleSubmit)}
        className="flex flex-col gap-4 p-6 max-w-2xl"
      >
        <HoldingsArray />
        <FormField name="start" label="Start" inline>
          <DateField />
        </FormField>
        <FormField name="end" label="End" inline>
          <DateField />
        </FormField>
        <FormField name="initial" label="Initial $" inline>
          <NumberField prefix="$" />
        </FormField>
        <FormField name="rebalance" label="Rebalance" inline>
          <SelectField options={REBALANCE_OPTIONS} />
        </FormField>
        <Button type="submit">Run Backtest</Button>
      </form>
    </FormProvider>
  );
}
```

**Tradeoffs:**

- The form is reset on URL change rather than on every render. `useFormSearchSync` checks search via `JSON.stringify` deep-equal so identical search payloads don't trigger reset.
- `keepDirtyValues: true` preserves in-progress edits when the URL changes underneath. If the user is mid-edit and an external navigation happens (rare — mostly browser back/forward), their typing isn't lost.
- The submit handler pushes to URL but does NOT directly trigger the backtest. The Tauri command runs from the URL-driven query (`useTauriQuery` keyed on the search). This means hitting back/forward replays previous results from cache and bookmarked URLs reproduce results without an extra round-trip.

---

## 6. Field Arrays — Repeating Holdings

Backtest config takes 1-50 holdings. RHF's `useFieldArray` manages dynamic lists.

### File: `apps/desktop/src/routes/analysis/components/holdings-array.tsx`

```typescript
import { useFieldArray, useFormContext } from "react-hook-form";
import {
  FormField,
  NumberField,
  TextField,
} from "@chrdfin/ui/components/form";
import { Button } from "@chrdfin/ui/components/button";
import { X, Plus } from "lucide-react";
import type { BacktestConfig } from "@chrdfin/types";

export function HoldingsArray(): JSX.Element {
  const { control } = useFormContext<BacktestConfig>();
  const { fields, append, remove } = useFieldArray({
    control,
    name: "holdings",
  });

  return (
    <div className="flex flex-col gap-2">
      <div className="text-xs text-muted-foreground uppercase">Holdings</div>
      <div className="flex flex-col gap-1">
        {fields.map((field, index) => (
          <div
            key={field.id}
            className="grid grid-cols-[1fr_120px_28px] gap-2 items-start"
          >
            <FormField name={`holdings.${index}.symbol`}>
              <TextField
                density="compact"
                placeholder="Ticker"
                style={{ textTransform: "uppercase" }}
              />
            </FormField>
            <FormField name={`holdings.${index}.weight`}>
              <NumberField density="compact" suffix="%" />
            </FormField>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => remove(index)}
              className="h-7 w-7 p-0"
              disabled={fields.length === 1}
              aria-label="Remove holding"
            >
              <X className="size-3" />
            </Button>
          </div>
        ))}
      </div>
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={() => append({ symbol: "", weight: 0 })}
        disabled={fields.length >= 50}
        className="self-start mt-1"
      >
        <Plus className="size-3 mr-1" />
        Add Holding
      </Button>
    </div>
  );
}
```

**Tradeoffs:**

- `useFieldArray` adds a stable `id` to each field for React keys (separate from the underlying data). Use `field.id` not `index` for keys.
- The "remove" button is disabled when only one holding remains — the schema requires at least one. Alternatively, the schema could allow zero and the form could handle the empty state, but a minimum-of-one is the saner UX.
- Weight is captured as percent in the UI but the schema expects 0-1 fraction. Either change the schema to accept 0-100 (more user-friendly) or transform at submit time. For chrdfin we keep the schema in fractions and do the conversion in the consumer (backtest config form) to keep the schema aligned with how the Rust backend expects values.

Actually: for that mismatch, the cleanest approach is a separate "form schema" from the "wire schema":

```typescript
// Form schema (UI-friendly: percentages 0-100)
const HoldingFormSchema = z.object({
  symbol: z.string()...,
  weightPercent: z.number().min(0).max(100),
});

// Wire schema (Rust-aligned: fractions 0-1)
const HoldingWireSchema = z.object({
  symbol: z.string()...,
  weight: z.number().min(0).max(1),
});

// Adapter
function toWire(form: HoldingFormInput): HoldingWireInput {
  return { symbol: form.symbol, weight: form.weightPercent / 100 };
}
```

This is how `@chrdfin/types` should structure shared schemas — paired form/wire types where they diverge. Document the divergence in the shared types file.

---

## 7. Async Validation: Ticker Autocomplete

Some validations need backend round-trips (does this ticker exist in our database?). RHF supports async validators via `validate` in `useController` or via Zod's `refine`. For chrdfin we prefer a separate combobox component over inline async validation — the user picks from valid options rather than typing freeform and getting an error.

### File: `packages/@chrdfin/ui/src/components/form/ticker-combobox.tsx`

```typescript
import { useController, useFormContext } from "react-hook-form";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Check, ChevronsUpDown } from "lucide-react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "../command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "../popover";
import { Button } from "../button";
import { useFormField } from "./form";
import { cn } from "../../lib/utils";

interface TickerSuggestion {
  symbol: string;
  name: string;
}

export interface TickerComboboxProps {
  density?: "default" | "compact";
}

/**
 * Ticker selector with debounced async autocomplete.
 *
 * Queries the Tauri `search_tickers` command which returns symbol+name
 * suggestions from the local DuckDB ticker metadata.
 */
export function TickerCombobox({
  density = "default",
}: TickerComboboxProps): JSX.Element {
  const { control } = useFormContext();
  const { name, id, hasError } = useFormField();
  const { field } = useController({ name, control });
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");

  const { data: suggestions = [] } = useQuery<TickerSuggestion[]>({
    queryKey: ["ticker-search", query],
    queryFn: () => invoke("search_tickers", { query, limit: 12 }),
    enabled: query.length >= 1,
    staleTime: 60 * 1000,
  });

  const heightClass = density === "compact" ? "h-7" : "h-8";

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id={id}
          type="button"
          variant="outline"
          role="combobox"
          aria-expanded={open}
          aria-invalid={hasError || undefined}
          className={cn(
            "w-full justify-between font-mono",
            heightClass,
            hasError && "border-destructive",
          )}
        >
          {field.value || <span className="text-muted-foreground">Select ticker</span>}
          <ChevronsUpDown className="ml-2 size-3 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-72 p-0" align="start">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search tickers..."
            value={query}
            onValueChange={setQuery}
          />
          <CommandList>
            <CommandEmpty>No matches.</CommandEmpty>
            <CommandGroup>
              {suggestions.map((s) => (
                <CommandItem
                  key={s.symbol}
                  value={s.symbol}
                  onSelect={() => {
                    field.onChange(s.symbol);
                    setOpen(false);
                  }}
                  className="font-mono text-xs"
                >
                  <Check
                    className={cn(
                      "mr-2 size-3",
                      field.value === s.symbol ? "opacity-100" : "opacity-0",
                    )}
                  />
                  <span className="font-medium mr-2">{s.symbol}</span>
                  <span className="text-muted-foreground truncate">{s.name}</span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
```

**Tradeoffs:**

- Async query is 1-character minimum. Lower threshold = more queries; higher = laggier UX. 1 char is right because the local DuckDB query is fast (<5ms) and users often start with a single character and pause.
- `shouldFilter={false}` on `<Command>` because the backend already filters. Without this, the cmdk library does its own client-side filtering on top.
- Stale time 60s — if the user opens the combobox twice within a minute with the same query prefix, no extra round-trip.
- This component handles the validation implicitly: only valid tickers can be selected. Inline async validation (red border on free text) would require letting users type invalid tickers and showing an error, which is worse UX than constraining to valid options upfront.

---

## 8. Filter Bar Pattern (URL-only, no form state)

Screener filters don't need form state — they're a URL-driven UI. The pattern is to skip RHF entirely and bind controls directly to search params.

### File: `apps/desktop/src/routes/market/components/screener-filters.tsx`

```typescript
import { useScreenerNav } from "@/hooks/use-domain-nav";
import { Button } from "@chrdfin/ui/components/button";
import { cn } from "@chrdfin/ui/lib/utils";

const ASSET_OPTIONS = [
  { value: "stocks", label: "Stocks" },
  { value: "etfs", label: "ETFs" },
  { value: "all", label: "All" },
] as const;

const SECTOR_OPTIONS = [
  "Tech", "Healthcare", "Financials", "Energy",
  "Consumer", "Industrials", "Materials", "Utilities",
];

export function ScreenerFilters(): JSX.Element {
  const { search, updateSearch, resetFilters } = useScreenerNav();

  return (
    <div className="flex items-center gap-2 px-6 h-14 border-b border-border">
      <FilterPill label="Asset">
        <select
          value={search.asset ?? "stocks"}
          onChange={(e) => updateSearch({ asset: e.target.value as typeof ASSET_OPTIONS[number]["value"] })}
          className="bg-transparent text-xs border-0 focus:outline-none"
        >
          {ASSET_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>{o.label}</option>
          ))}
        </select>
      </FilterPill>

      <FilterPill label="Market Cap">
        <RangeMini
          min={search.marketCapMin}
          max={search.marketCapMax}
          onChange={(min, max) =>
            updateSearch({ marketCapMin: min, marketCapMax: max })
          }
          formatter={(v) => `$${(v / 1e9).toFixed(0)}B`}
        />
      </FilterPill>

      <FilterPill label="Yield">
        <NumberMini
          value={search.yieldMin}
          onChange={(v) => updateSearch({ yieldMin: v })}
          suffix="%"
          placeholder=">"
        />
      </FilterPill>

      <Button
        variant="ghost"
        size="sm"
        onClick={resetFilters}
        className="text-xs text-muted-foreground"
      >
        Clear filters
      </Button>

      <div className="ml-auto text-xs text-muted-foreground">
        {/* Result count populated by parent route */}
      </div>
    </div>
  );
}

function FilterPill({
  label,
  children,
}: { label: string; children: React.ReactNode }): JSX.Element {
  return (
    <div className="flex items-center gap-1.5 h-7 px-2 border border-border rounded-sm">
      <span className="text-xs text-muted-foreground">{label}</span>
      {children}
    </div>
  );
}

// RangeMini and NumberMini omitted for brevity — they're trivial
// uncontrolled inputs that call onChange on blur/Enter.
```

**Tradeoffs:**

- No RHF here. The filter bar has no submit step, no validation across multiple fields, and no save-state semantics. URL is the entirety of the state. Adding a form layer would just be ceremony.
- Updates push to URL on each change. Consumers (the screener results table) read from search and refetch automatically. The result count and the filter bar share a single source of truth.
- Reset is a hard navigate to `{}` — drops all filters. The schema's defaults reapply on next render.

---

## 9. Submit + Mutation Integration

Forms that persist data tie into TanStack Query mutations. The pattern: form's `onSubmit` calls `mutate()`, success triggers cache invalidation per `data-fetching-patterns.md`.

### Reference: Transaction entry form

```typescript
// apps/desktop/src/routes/tracking/components/transaction-form.tsx
import { Form, FormField, NumberField, DateField, SelectField } from "@chrdfin/ui";
import { TickerCombobox } from "@chrdfin/ui/components/form/ticker-combobox";
import { Button } from "@chrdfin/ui";
import { useAddTransaction } from "../hooks/use-add-transaction";
import {
  TransactionInputSchema,
  type TransactionInput,
} from "@chrdfin/types";

const TYPE_OPTIONS = [
  { value: "buy", label: "Buy" },
  { value: "sell", label: "Sell" },
  { value: "dividend", label: "Dividend" },
  { value: "split", label: "Split" },
  { value: "transfer", label: "Transfer" },
] as const;

export function TransactionForm({
  portfolioId,
  onComplete,
}: {
  portfolioId: string;
  onComplete: () => void;
}): JSX.Element {
  const addTransaction = useAddTransaction(portfolioId);

  return (
    <Form
      schema={TransactionInputSchema}
      defaultValues={{
        portfolioId,
        symbol: "",
        type: "buy",
        date: new Date().toISOString().slice(0, 10),
        shares: 0,
        price: 0,
        fees: 0,
        notes: "",
      }}
      onSubmit={(values) =>
        addTransaction.mutate(values, {
          onSuccess: () => onComplete(),
        })
      }
      className="grid grid-cols-2 gap-3 max-w-xl"
    >
      <FormField name="symbol" label="Ticker">
        <TickerCombobox />
      </FormField>
      <FormField name="type" label="Type">
        <SelectField options={TYPE_OPTIONS} />
      </FormField>
      <FormField name="date" label="Date">
        <DateField max={new Date().toISOString().slice(0, 10)} />
      </FormField>
      <FormField name="shares" label="Shares">
        <NumberField />
      </FormField>
      <FormField name="price" label="Price">
        <NumberField prefix="$" />
      </FormField>
      <FormField name="fees" label="Fees">
        <NumberField prefix="$" />
      </FormField>

      {addTransaction.isError ? (
        <div className="col-span-2 text-xs text-destructive">
          {addTransaction.error.message}
        </div>
      ) : null}

      <div className="col-span-2 flex gap-2 pt-2">
        <Button type="submit" disabled={addTransaction.isPending}>
          {addTransaction.isPending ? "Saving..." : "Add Transaction"}
        </Button>
        <Button type="button" variant="ghost" onClick={onComplete}>
          Cancel
        </Button>
      </div>
    </Form>
  );
}
```

**Tradeoffs:**

- `addTransaction.mutate(values, { onSuccess: ... })` — per-call success handler in addition to the mutation hook's invalidation logic. The hook handles cache invalidation; the per-call handler closes the modal/drawer.
- Submit button shows "Saving..." during the mutation. Adequate for chrdfin's terminal density; flashier patterns (spinner inside button) feel out of place.
- Date max set to today — prevents future-dated transactions, which would corrupt holdings calculations. Schema enforcement of "date <= today" would also work but UI-side disabling gives faster feedback.

---

## 10. Validation Triggers

| Mode | Use |
|---|---|
| `mode: "onSubmit"` | Multi-step wizards where validation only matters at boundary |
| `mode: "onBlur"` (default in `<Form>`) | Standard forms — feedback on field exit, not during typing |
| `mode: "onChange"` | Filter bars, calculators with live results, inline editing |
| `mode: "onTouched"` | Forms where users frequently leave fields empty intentionally |

Switch via the `mode` prop on `<Form>` or in the `useForm()` config. Default `onBlur` is the right choice for most chrdfin forms.

For revalidation after first submit, set `reValidateMode: "onChange"` so once an error appears, the user sees it disappear immediately when they fix it.

---

## 11. Common Pitfalls

| Symptom | Cause | Fix |
|---|---|---|
| Form value stays as `""` instead of `undefined` | NumberField not converting empty string | Verify `NumberField` handles `""` → `undefined` in onChange (per recipe) |
| Zod error message shows "Expected number, received string" | Form value is a string but schema expects number | Coerce in the field component, not in the schema. Schema defines the shape; the field bridges UI types to schema types |
| Submit button stays disabled even though form is valid | `disabled={!formState.isValid}` and `formState.isValid` defaults to false until first validation | Either submit and let the resolver validate, or set `mode: "onChange"` so isValid updates as the user types |
| Form doesn't reset after successful submit | RHF doesn't auto-reset | Call `form.reset()` in the mutation's `onSuccess` |
| Field arrays render duplicate keys | Using `index` as key | Use `field.id` (provided by `useFieldArray`) |
| Cross-field validation error attaches to wrong field | `path: []` on `superRefine` issue | Add specific path: `path: ["fieldName"]` so error appears next to the relevant control |
| Date field stores date as `Date` object | RHF preserves the raw input value | Always store dates as ISO strings in form state to match Zod schemas and Tauri command inputs |
| Search-synced form clobbers in-progress edits | `useFormSearchSync` resetting on every render | Verify `keepDirtyValues: true` in the reset call; check the dependency array uses `JSON.stringify` |
| Async validation never fires | Zod's `refine` returns `boolean` not Promise | Use `.refine(async (v) => ...)` for async, or move validation outside the schema (e.g. ticker combobox limits to valid values only) |
| Combobox query refires on every keystroke without debounce | Query key including `query` string changes per character | This is fine if the query is cheap; for expensive queries, debounce `query` state via `useDebouncedValue` before passing to the queryKey |

---

## 12. Testing Patterns

### Schema tests

Schema tests live alongside the schemas in `@chrdfin/types`:

```typescript
import { describe, it, expect } from "vitest";
import {
  CompoundInterestSchema,
  BacktestConfigSchema,
} from "./forms";

describe("BacktestConfigSchema", () => {
  it("accepts a valid 60/40 portfolio", () => {
    const result = BacktestConfigSchema.safeParse({
      holdings: [
        { symbol: "SPY", weight: 0.6 },
        { symbol: "AGG", weight: 0.4 },
      ],
      start: "2005-01-01",
      end: "2024-12-31",
    });
    expect(result.success).toBe(true);
  });

  it("rejects weights that don't sum to 1.0", () => {
    const result = BacktestConfigSchema.safeParse({
      holdings: [
        { symbol: "SPY", weight: 0.5 },
        { symbol: "AGG", weight: 0.4 },
      ],
      start: "2005-01-01",
      end: "2024-12-31",
    });
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].path).toContain("holdings");
    }
  });

  it("rejects end date <= start date", () => {
    const result = BacktestConfigSchema.safeParse({
      holdings: [{ symbol: "SPY", weight: 1 }],
      start: "2024-01-01",
      end: "2023-12-31",
    });
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues.some((i) => i.path.includes("end"))).toBe(true);
    }
  });
});
```

### Form component tests

Test the form with React Testing Library. RHF runs the resolver synchronously on submit, so these tests don't need extensive `waitFor`.

```typescript
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CompoundInterestPage } from "./compound-interest.lazy";

describe("CompoundInterestPage", () => {
  it("computes result on valid submit", async () => {
    const user = userEvent.setup();
    render(<CompoundInterestPage />);

    // Defaults are pre-filled; just submit
    await user.click(screen.getByRole("button", { name: /calculate/i }));

    expect(await screen.findByText(/Final Amount/i)).toBeInTheDocument();
  });

  it("shows validation errors on invalid input", async () => {
    const user = userEvent.setup();
    render(<CompoundInterestPage />);

    const yearsInput = screen.getByLabelText(/years/i);
    await user.clear(yearsInput);
    await user.tab(); // blur to trigger validation

    expect(await screen.findByText(/required/i)).toBeInTheDocument();
  });
});
```

### Mutation integration tests

Mock `useTauriMutation` to assert the mutation is called with correctly-shaped data. See `data-fetching-patterns.md` section 14 for the Tauri mocking pattern.

---

## 13. References

- React Hook Form docs: <https://react-hook-form.com/>
- React Hook Form + Zod resolver: <https://github.com/react-hook-form/resolvers>
- Zod docs: <https://zod.dev/>
- shadcn/ui Form primitive: <https://ui.shadcn.com/docs/components/form>
- WCAG form accessibility: <https://www.w3.org/WAI/tutorials/forms/>

---

## 14. Document Maintenance

When adding a new form:

1. Define the schema in `@chrdfin/types/src/forms.ts`.
2. Compose the form using existing field primitives. Add a new field primitive only if no existing one fits and the use case will recur in 2+ forms.
3. Wire submit to a mutation hook from the per-domain hooks directory.
4. Add a schema test covering valid input, each constraint, and at least one cross-field rule.
5. Add a route-level test rendering the form and asserting end-to-end behavior for the happy path.

When adding a new field primitive:

1. Place it under `packages/@chrdfin/ui/src/components/form/`.
2. Reuse `useFormContext` + `useFormField` for wiring.
3. Forward `aria-describedby` and `aria-invalid` for accessibility.
4. Match density variants (`h-7`/`h-8`) used by existing primitives.
5. Document in this file with a short example.

When changing a schema:

1. Schema lives in `@chrdfin/types`. Bumping it requires considering: form callers, search param parsers (per `route-conventions.md`), and Tauri command serde validation on the Rust side.
2. Add migration considerations to the database-schema-reference if any persisted data follows the schema shape.
