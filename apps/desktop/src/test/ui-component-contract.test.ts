import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const componentDirectory = join(process.cwd(), "src", "components", "ui");
const requiredComponents = [
  { exportName: "Accordion", fileName: "accordion.tsx" },
  { exportName: "Alert", fileName: "alert.tsx" },
  { exportName: "AlertDialog", fileName: "alert-dialog.tsx" },
  { exportName: "AspectRatio", fileName: "aspect-ratio.tsx" },
  { exportName: "Avatar", fileName: "avatar.tsx" },
  { exportName: "Badge", fileName: "badge.tsx" },
  { exportName: "Breadcrumb", fileName: "breadcrumb.tsx" },
  { exportName: "Button", fileName: "button.tsx" },
  { exportName: "ButtonGroup", fileName: "button-group.tsx" },
  { exportName: "Calendar", fileName: "calendar.tsx" },
  { exportName: "Card", fileName: "card.tsx" },
  { exportName: "Carousel", fileName: "carousel.tsx" },
  { exportName: "Chart", fileName: "chart.tsx" },
  { exportName: "Checkbox", fileName: "checkbox.tsx" },
  { exportName: "Collapsible", fileName: "collapsible.tsx" },
  { exportName: "Combobox", fileName: "combobox.tsx" },
  { exportName: "Command", fileName: "command.tsx" },
  { exportName: "ContextMenu", fileName: "context-menu.tsx" },
  { exportName: "DataTable", fileName: "data-table.tsx" },
  { exportName: "DatePicker", fileName: "date-picker.tsx" },
  { exportName: "Dialog", fileName: "dialog.tsx" },
  { exportName: "Direction", fileName: "direction.tsx" },
  { exportName: "Drawer", fileName: "drawer.tsx" },
  { exportName: "DropdownMenu", fileName: "dropdown-menu.tsx" },
  { exportName: "Empty", fileName: "empty.tsx" },
  { exportName: "Field", fileName: "field.tsx" },
  { exportName: "HoverCard", fileName: "hover-card.tsx" },
  { exportName: "Input", fileName: "input.tsx" },
  { exportName: "InputGroup", fileName: "input-group.tsx" },
  { exportName: "InputOTP", fileName: "input-otp.tsx" },
  { exportName: "Item", fileName: "item.tsx" },
  { exportName: "Kbd", fileName: "kbd.tsx" },
  { exportName: "Label", fileName: "label.tsx" },
  { exportName: "Menubar", fileName: "menubar.tsx" },
  { exportName: "NativeSelect", fileName: "native-select.tsx" },
  { exportName: "NavigationMenu", fileName: "navigation-menu.tsx" },
  { exportName: "Pagination", fileName: "pagination.tsx" },
  { exportName: "Popover", fileName: "popover.tsx" },
  { exportName: "Progress", fileName: "progress.tsx" },
  { exportName: "RadioGroup", fileName: "radio-group.tsx" },
  { exportName: "Resizable", fileName: "resizable.tsx" },
  { exportName: "ScrollArea", fileName: "scroll-area.tsx" },
  { exportName: "Select", fileName: "select.tsx" },
  { exportName: "Separator", fileName: "separator.tsx" },
  { exportName: "Sheet", fileName: "sheet.tsx" },
  { exportName: "Sidebar", fileName: "sidebar.tsx" },
  { exportName: "Skeleton", fileName: "skeleton.tsx" },
  { exportName: "Slider", fileName: "slider.tsx" },
  { exportName: "Sonner", fileName: "sonner.tsx" },
  { exportName: "Spinner", fileName: "spinner.tsx" },
  { exportName: "Switch", fileName: "switch.tsx" },
  { exportName: "Table", fileName: "table.tsx" },
  { exportName: "Tabs", fileName: "tabs.tsx" },
  { exportName: "Textarea", fileName: "textarea.tsx" },
  { exportName: "Toast", fileName: "toast.tsx" },
  { exportName: "Toggle", fileName: "toggle.tsx" },
  { exportName: "ToggleGroup", fileName: "toggle-group.tsx" },
  { exportName: "Tooltip", fileName: "tooltip.tsx" },
  { exportName: "Typography", fileName: "typography.tsx" },
] as const;

describe("UI component contract", () => {
  it("has only required primitive component files", () => {
    const files = readdirSync(componentDirectory).filter((fileName) => fileName.endsWith(".tsx"));
    expect(files.sort()).toEqual(requiredComponents.map((component) => component.fileName).sort());
  });

  it("exports every required primitive", () => {
    const source = readdirSync(componentDirectory).filter((fileName) => fileName.endsWith(".tsx")).map((fileName) => readFileSync(join(componentDirectory, fileName), "utf8")).join("\n");
    const exportedNames = new Set([
      ...[...source.matchAll(/export function\s+(\w+)/g)].map((match) => match[1]),
      ...[...source.matchAll(/export const\s+(\w+)/g)].map((match) => match[1]),
    ]);
    const missingExports = requiredComponents.map((component) => component.exportName).filter((exportName) => !exportedNames.has(exportName));
    expect(missingExports).toEqual([]);
  });
});
