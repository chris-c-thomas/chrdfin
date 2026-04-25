import { useEffect, useState } from "react";

export type MarketStatus =
  | "open"
  | "pre-market"
  | "after-market"
  | "closed"
  | "weekend"
  | "holiday";

/**
 * NYSE full-day market holidays. Update annually each December.
 * Source: https://www.nyse.com/markets/hours-calendars
 */
const NYSE_HOLIDAYS_2026: ReadonlySet<string> = new Set([
  "2026-01-01", // New Year's Day
  "2026-01-19", // MLK Day
  "2026-02-16", // Presidents' Day
  "2026-04-03", // Good Friday
  "2026-05-25", // Memorial Day
  "2026-06-19", // Juneteenth
  "2026-07-03", // Independence Day (observed)
  "2026-09-07", // Labor Day
  "2026-11-26", // Thanksgiving
  "2026-12-25", // Christmas
]);

interface MarketStatusValue {
  status: MarketStatus;
  etTime: string;
}

function computeStatus(now: Date): MarketStatusValue {
  // Render the current instant in NY local time using Intl, then read
  // its components back. Avoids hand-rolling DST math.
  const fmt = new Intl.DateTimeFormat("en-US", {
    timeZone: "America/New_York",
    weekday: "short",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
  const parts = fmt.formatToParts(now);
  const get = (type: string) => parts.find((p) => p.type === type)?.value ?? "";
  const weekday = get("weekday");
  const year = get("year");
  const month = get("month");
  const day = get("day");
  const hourRaw = get("hour");
  const minute = get("minute");
  // Intl returns "24" at midnight in some locales; normalize to "00".
  const hour = hourRaw === "24" ? "00" : hourRaw;

  const isoDate = `${year}-${month}-${day}`;
  const minutes = Number(hour) * 60 + Number(minute);
  const etTime = `${hour}:${minute}`;

  if (weekday === "Sat" || weekday === "Sun") {
    return { status: "weekend", etTime };
  }
  if (NYSE_HOLIDAYS_2026.has(isoDate)) {
    return { status: "holiday", etTime };
  }

  const PRE = 4 * 60;
  const OPEN = 9 * 60 + 30;
  const CLOSE = 16 * 60;
  const AFTER_END = 20 * 60;

  if (minutes < PRE) return { status: "closed", etTime };
  if (minutes < OPEN) return { status: "pre-market", etTime };
  if (minutes < CLOSE) return { status: "open", etTime };
  if (minutes < AFTER_END) return { status: "after-market", etTime };
  return { status: "closed", etTime };
}

export function useMarketStatus(): MarketStatusValue {
  const [value, setValue] = useState<MarketStatusValue>(() => computeStatus(new Date()));

  useEffect(() => {
    const id = window.setInterval(() => {
      setValue(computeStatus(new Date()));
    }, 1000);
    return () => window.clearInterval(id);
  }, []);

  return value;
}
