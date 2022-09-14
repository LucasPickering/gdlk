import { Currency } from "./types";

/**
 * Format a currency value as a pretty string.
 */
export function formatCurrency(currency: Currency): string {
  return `${currency}ƒ`;
}
