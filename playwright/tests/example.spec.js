// @ts-check
import { test, expect } from "@playwright/test";

test.describe("Example tests", () => {
  test("has title", async ({ page }) => {
    await page.goto("/");

    // Expect a title to exist
    await expect(page).toHaveTitle(/.+/);
  });

  test("page loads without console errors", async ({ page }) => {
    const errors = [];

    page.on("console", (msg) => {
      if (msg.type() === "error") {
        errors.push(msg.text());
      }
    });

    await page.goto("/");
    await page.waitForLoadState("networkidle");

    expect(errors).toHaveLength(0);
  });
});
