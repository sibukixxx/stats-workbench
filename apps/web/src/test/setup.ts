import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/preact";
import { afterEach, beforeEach } from "vitest";
import { resetAppState } from "../state";

beforeEach(() => {
  resetAppState("/");
});

afterEach(() => {
  cleanup();
});
