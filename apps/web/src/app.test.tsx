import { fireEvent, render, screen } from "@testing-library/preact";
import { describe, expect, it } from "vitest";
import { App } from "./app";

describe("Stats Workbench web dashboard", () => {
  it("discloses the sample-only demo on every screen", () => {
    render(<App />);
    expect(screen.getByRole("note")).toHaveTextContent("Rust CLI は実行しません");
    fireEvent.click(screen.getAllByRole("button", { name: /Summary/i })[0]);
    expect(screen.getByRole("note")).toHaveTextContent("顧客情報や秘密情報は入力しないでください");
  });

  it("runs summary, correlation, outlier and recipe sample previews", () => {
    render(<App />);
    for (const [menu, action] of [
      [/Summary/i, "要約統計をモック実行"],
      [/Correlation/i, "相関分析をモック実行"],
      [/Outlier/i, "外れ値検出をモック実行"],
      [/Recipe Run/i, "レシピをモック実行"],
    ] as const) {
      fireEvent.click(screen.getAllByRole("button", { name: menu })[0]);
      fireEvent.click(screen.getByRole("button", { name: action }));
      expect(screen.getByText(/mock executed at/)).toBeInTheDocument();
    }
  });

  it("renders the overview screen with Japanese menu descriptions", () => {
    render(<App />);

    expect(screen.getByText("分析メニュー全体を俯瞰するダッシュボード")).toBeInTheDocument();
    expect(screen.getByText("要約統計画面")).toBeInTheDocument();
    expect(screen.getByText("外れ値検出画面")).toBeInTheDocument();
  });

  it("navigates to the correlation screen from the sidebar", () => {
    render(<App />);

    fireEvent.click(screen.getAllByRole("button", { name: /Correlation/i })[0]);

    expect(screen.getByText("相関分析画面")).toBeInTheDocument();
    expect(screen.getByLabelText("Method")).toBeInTheDocument();
    expect(screen.getByText("Correlation Matrix")).toBeInTheDocument();
  });

  it("shows a validation error when sqlite source misses table and query", () => {
    render(<App />);

    fireEvent.click(screen.getAllByRole("button", { name: /Summary/i })[0]);
    fireEvent.click(screen.getByRole("button", { name: "SQLite" }));

    fireEvent.input(screen.getByLabelText("Table"), { target: { value: "" } });
    fireEvent.input(screen.getByLabelText("Query"), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "要約統計をモック実行" }));

    expect(
      screen.getByText("SQLite ソースでは table か query のどちらかが必要です。"),
    ).toBeInTheDocument();
  });

  it("renders distribution bins after execution", () => {
    render(<App />);

    fireEvent.click(screen.getAllByRole("button", { name: /Distribution/i })[0]);
    fireEvent.input(screen.getByLabelText("Column"), { target: { value: "price" } });
    fireEvent.input(screen.getByLabelText("Bins"), { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "分布確認をモック実行" }));

    expect(screen.getByText(/price \/ 5 bins/)).toBeInTheDocument();
    expect(screen.getByText(/Bin 1:/)).toBeInTheDocument();
  });

  it("renders recipe execution artifacts", () => {
    render(<App />);

    fireEvent.click(screen.getAllByRole("button", { name: /Recipe Run/i })[0]);

    expect(screen.getByText("Recipe Execution Preview")).toBeInTheDocument();
    expect(screen.getByText("summary")).toBeInTheDocument();
    expect(screen.getByText("outliers")).toBeInTheDocument();
  });
});
