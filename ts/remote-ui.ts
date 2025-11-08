/**
 * High-level TypeScript client for the `RemoteUiHost` RPC defined in `src/ui/remote.rs`.
 *
 * The API mirrors the commands consumed by the Rust side and wraps them in typed, well
 * documented helpers. Use this module to create UI widgets, update their state and work
 * with layouts without manually hand-crafting JSON payloads.
 *
 * @example
 * ```ts
 * const ui = new RemoteUiClient();
 * const panel = await ui.panel('settings_panel', {
 *   position: [16, 32],
 *   size: [280, 200],
 *   title: 'Settings',
 * });
 * const save = await ui.button('save_button', { label: 'Save' });
 * await save.attachTo(panel, [24, 48]);
 * await save.setSize([140, 36]);
 * await save.setColors({
 *   normal: '#3498db',
 *   hover: '#5dade2',
 *   pressed: '#2e86c1',
 * });
 * ```
 */

/* eslint-disable @typescript-eslint/no-redeclare */

declare const Ui: {
  sendOverlayCommand(command: string): Promise<void>;
};

declare function __registerUiCallback(widgetId: string, eventType: string, callback: (event: any) => void): void;
declare function __unregisterUiCallback(widgetId: string, eventType: string): void;

type OverlayCommandSender = (command: string) => void | Promise<void>;

interface RemoteCommand<TParams = unknown> {
  id: string;
  method: string;
  params?: TParams;
}

interface Vec2 {
  x: number;
  y: number;
}

interface Size {
  width: number;
  height: number;
}

interface ColorPayload {
  r: number;
  g: number;
  b: number;
  a: number;
}

export type Vec2Like = { x: number; y: number } | readonly [number, number];
export type SizeLike = { width: number; height: number } | readonly [number, number];
export type PaddingLike = Vec2Like;

export type ColorLike =
  | { r: number; g: number; b: number; a?: number }
  | readonly [number, number, number]
  | readonly [number, number, number, number]
  | string;

export type CrossAlignment = 'start' | 'center' | 'end' | 'centre';

export type PaletteSlot =
  | 'text_primary'
  | 'text_secondary'
  | 'surface_dark'
  | 'surface'
  | 'surface_light'
  | 'accent'
  | 'accent_soft'
  | 'border_soft'
  | 'border_subtle'
  | 'checkmark'
  | 'shadow';

export interface PaletteOverrides {
  textPrimary?: ColorLike;
  textSecondary?: ColorLike;
  surfaceDark?: ColorLike;
  surface?: ColorLike;
  surfaceLight?: ColorLike;
  accent?: ColorLike;
  accentSoft?: ColorLike;
  borderSoft?: ColorLike;
  borderSubtle?: ColorLike;
  checkmark?: ColorLike;
  shadow?: ColorLike;
}

interface BaseWidgetOptions {
  position?: Vec2Like;
  size?: SizeLike;
}

export interface ButtonInit extends BaseWidgetOptions {
  label?: string;
  text?: string;
}

export interface CheckboxInit extends BaseWidgetOptions {
  label?: string;
  checked?: boolean;
}

export interface LabelInit extends BaseWidgetOptions {
  text?: string;
  color?: ColorLike;
}

export interface TextBoxInit extends BaseWidgetOptions {
  text?: string;
  placeholder?: string;
}

export interface DropdownInit extends BaseWidgetOptions {
  options?: Iterable<string>;
  placeholder?: string | null;
  maxVisibleItems?: number;
  optionHeight?: number;
  selectedIndex?: number;
}

export interface PanelInit extends BaseWidgetOptions {
  title?: string;
}

export interface LayoutInit {
  position?: Vec2Like;
}

export interface ButtonColors {
  normal: ColorLike;
  hover: ColorLike;
  pressed: ColorLike;
}

export interface PanelColors {
  background: ColorLike;
  titleBar: ColorLike;
}

export interface RemoteUiClientOptions {
  /**
   * Identifier used for host-level commands such as `clear_all` and `set_palette`.
   * Defaults to `_host`, which matches the expectations in the Rust tests.
   */
  hostId?: string;
  /**
   * Custom sender, useful for testing. Defaults to `Ui.sendOverlayCommand`.
   */
  sender?: OverlayCommandSender;
}

type WidgetKind =
  | 'button'
  | 'checkbox'
  | 'label'
  | 'textbox'
  | 'dropdown'
  | 'panel'
  | 'horizontal_layout'
  | 'vertical_layout';

export interface ButtonClickEvent {
  type: 'click';
  id: string;
  label: string;
}

export interface CheckboxToggleEvent {
  type: 'toggle';
  id: string;
  label: string;
  checked: boolean;
}

export interface TextChangeEvent {
  type: 'change';
  id: string;
  text: string;
}

export interface TextBoxFocusEvent {
  type: 'focus';
  id: string;
  focused: boolean;
}

export interface DropdownChangeEvent {
  type: 'change';
  id: string;
  selected: string;
}

export interface PanelDragStartEvent {
  type: 'dragstart';
  id: string;
}

export interface PanelDragEvent {
  type: 'drag';
  id: string;
  position: Vec2;
}

export interface PanelDragEndEvent {
  type: 'dragend';
  id: string;
}

export interface PanelToggleChangeEvent {
  type: 'togglechange';
  id: string;
  collapsed: boolean;
}

const DEFAULT_HOST_ID = '_host';
const HEX_COLOR_REGEX = /^#([0-9a-f]{6}|[0-9a-f]{8})$/i;

const paletteKeyMap: Record<keyof PaletteOverrides, string> = {
  textPrimary: 'text_primary',
  textSecondary: 'text_secondary',
  surfaceDark: 'surface_dark',
  surface: 'surface',
  surfaceLight: 'surface_light',
  accent: 'accent',
  accentSoft: 'accent_soft',
  borderSoft: 'border_soft',
  borderSubtle: 'border_subtle',
  checkmark: 'checkmark',
  shadow: 'shadow',
};

const defaultSender: OverlayCommandSender = async (command) => {
  if (typeof Ui === 'undefined' || typeof Ui.sendOverlayCommand !== 'function') {
    throw new Error('Ui.sendOverlayCommand is not available. Provide a custom sender to RemoteUiClient.');
  }
  await Ui.sendOverlayCommand(command);
};

function resolveId(target: string | RemoteWidgetBase): string {
  return typeof target === 'string' ? target : target.id;
}

function ensureFinite(value: number, label: string): number {
  if (!Number.isFinite(value)) {
    throw new Error(`${label} must be a finite number`);
  }
  return value;
}

function normalizeVec2(value: Vec2Like, label: string): Vec2 {
  if (Array.isArray(value)) {
    const [x, y] = value;
    return { x: ensureFinite(Number(x), `${label}.x`), y: ensureFinite(Number(y), `${label}.y`) };
  }
  if (typeof value === 'object' && value !== null && 'x' in value && 'y' in value) {
    return {
      x: ensureFinite(Number((value as { x: number }).x), `${label}.x`),
      y: ensureFinite(Number((value as { y: number }).y), `${label}.y`),
    };
  }
  throw new Error(`${label} must be a tuple [x, y] or an object with x/y properties`);
}

function normalizeSize(value: SizeLike, label: string): Size {
  if (Array.isArray(value)) {
    const [width, height] = value;
    return {
      width: ensureFinite(Number(width), `${label}.width`),
      height: ensureFinite(Number(height), `${label}.height`),
    };
  }
  if (typeof value === 'object' && value !== null && 'width' in value && 'height' in value) {
    return {
      width: ensureFinite(Number((value as { width: number }).width), `${label}.width`),
      height: ensureFinite(Number((value as { height: number }).height), `${label}.height`),
    };
  }
  throw new Error(`${label} must be a tuple [width, height] or an object with width/height properties`);
}

function normalizeColor(value: ColorLike, label: string): ColorPayload {
  if (typeof value === 'string') {
    return parseHexColor(value, label);
  }
  if (Array.isArray(value)) {
    const [r, g, b, a] = value;
    return toColorPayload(
      ensureFinite(Number(r), `${label}.r`),
      ensureFinite(Number(g), `${label}.g`),
      ensureFinite(Number(b), `${label}.b`),
      a !== undefined ? ensureFinite(Number(a), `${label}.a`) : 1,
      label,
    );
  }
  if (typeof value === 'object' && value !== null) {
    const { r, g, b, a } = value as { r: number; g: number; b: number; a?: number };
    return toColorPayload(
      ensureFinite(Number(r), `${label}.r`),
      ensureFinite(Number(g), `${label}.g`),
      ensureFinite(Number(b), `${label}.b`),
      a !== undefined ? ensureFinite(Number(a), `${label}.a`) : 1,
      label,
    );
  }
  throw new Error(`${label} must be a hex string, array, or object describing RGBA values`);
}

function toColorPayload(r: number, g: number, b: number, a: number, _label: string): ColorPayload {
  const normalizeChannel = (value: number): number => {
    if (value <= 0) {
      return 0;
    }
    if (value >= 1 && value <= 255) {
      return value > 1 ? value / 255 : value;
    }
    if (value > 255) {
      return 1;
    }
    return value;
  };

  const clamp = (value: number): number => Math.max(0, Math.min(value, 1));

  return {
    r: clamp(normalizeChannel(r)),
    g: clamp(normalizeChannel(g)),
    b: clamp(normalizeChannel(b)),
    a: clamp(normalizeChannel(a)),
  };
}

function parseHexColor(value: string, label: string): ColorPayload {
  if (!HEX_COLOR_REGEX.test(value)) {
    throw new Error(`${label} must be a hex string in the form #RRGGBB or #RRGGBBAA`);
  }
  const hex = value.slice(1);
  const r = parseInt(hex.slice(0, 2), 16);
  const g = parseInt(hex.slice(2, 4), 16);
  const b = parseInt(hex.slice(4, 6), 16);
  const a = hex.length === 8 ? parseInt(hex.slice(6, 8), 16) : 255;
  return {
    r: r / 255,
    g: g / 255,
    b: b / 255,
    a: a / 255,
  };
}

function normalizeBool(value: boolean, label: string): boolean {
  if (typeof value !== 'boolean') {
    throw new Error(`${label} must be a boolean`);
  }
  return value;
}

function normalizeOptionalString(value: string | null | undefined, label: string): string | null | undefined {
  if (value === undefined || value === null) {
    return value;
  }
  if (typeof value !== 'string') {
    throw new Error(`${label} must be a string or null`);
  }
  return value;
}

function normalizeString(value: string | undefined, label: string): string {
  if (value === undefined) {
    throw new Error(`${label} is required`);
  }
  if (typeof value !== 'string') {
    throw new Error(`${label} must be a string`);
  }
  return value;
}

function normalizeInteger(value: number, label: string): number {
  const coerced = ensureFinite(Number(value), label);
  if (!Number.isInteger(coerced) || coerced < 0) {
    throw new Error(`${label} must be a non-negative integer`);
  }
  return coerced;
}

function assignIfDefined(target: Record<string, unknown>, key: string, value: unknown): void {
  if (value !== undefined) {
    target[key] = value;
  }
}

function buildBasePayload(options?: BaseWidgetOptions): Record<string, unknown> {
  const payload: Record<string, unknown> = {};
  if (!options) {
    return payload;
  }
  if (options.position !== undefined) {
    payload.position = normalizeVec2(options.position, 'position');
  }
  if (options.size !== undefined) {
    payload.size = normalizeSize(options.size, 'size');
  }
  return payload;
}

class WidgetCommandBuilder {
  constructor(private readonly client: RemoteUiClient, private readonly id: string) {}

  async send(method: string, params?: unknown): Promise<void> {
    await this.client.sendWidgetCommand(this.id, method, params ?? {});
  }
}

export class RemoteUiClient {
  readonly hostId: string;
  private readonly sender: OverlayCommandSender;

  constructor(options: RemoteUiClientOptions = {}) {
    this.hostId = options.hostId ?? DEFAULT_HOST_ID;
    this.sender = options.sender ?? defaultSender;
  }

  /**
   * Removes every widget hosted by the remote UI.
   */
  async clearAll(): Promise<void> {
    await this.send({ id: this.hostId, method: 'clear_all' });
  }

  /**
   * Sets multiple palette colors in a single command. Keys correspond to the fields inside
   * `colors::Palette` (camelCase) and are mapped to snake_case for the RPC payload.
   */
  async setPalette(overrides: PaletteOverrides): Promise<void> {
    const payload: Record<string, unknown> = {};
    (Object.keys(paletteKeyMap) as Array<keyof PaletteOverrides>).forEach((key) => {
      const value = overrides[key];
      if (value !== undefined) {
        payload[paletteKeyMap[key]] = normalizeColor(value, `palette.${String(key)}`);
      }
    });
    if (Object.keys(payload).length === 0) {
      return;
    }
    await this.send({ id: this.hostId, method: 'set_palette', params: payload });
  }

  /**
   * Sets a single palette slot to a specific color.
   */
  async setPaletteSlot(slot: PaletteSlot, color: ColorLike): Promise<void> {
    await this.send({
      id: this.hostId,
      method: 'set_palette_slot',
      params: { slot, color: normalizeColor(color, 'palette.slot') },
    });
  }

  /**
   * Destroys a widget and detaches it from any parent.
   */
  async destroy(target: string | RemoteWidgetBase): Promise<void> {
    await this.send({ id: resolveId(target), method: 'destroy' });
  }

  /**
   * Attaches a child widget to a container (panel or layout). The parent receives the command,
   * mirroring the behavior exercised in `attach_child`.
   */
  async attachChild(parent: string | RemoteWidgetBase, child: string | RemoteWidgetBase, offset?: Vec2Like): Promise<void> {
    const params: Record<string, unknown> = { child: resolveId(child) };
    if (offset !== undefined) {
      params.offset = normalizeVec2(offset, 'offset');
    }
    await this.send({
      id: resolveId(parent),
      method: 'attach_child',
      params,
    });
  }

  /**
   * Registers a button widget and returns a strongly-typed handle for further updates.
   */
  async button(id: string, init: ButtonInit = {}): Promise<RemoteButton> {
    const payload = buildBasePayload(init);
    assignIfDefined(payload, 'label', init.label);
    assignIfDefined(payload, 'text', init.text);
    await this.createWidget(id, 'button', payload);
    return new RemoteButton(id, this);
  }

  async checkbox(id: string, init: CheckboxInit = {}): Promise<RemoteCheckbox> {
    const payload = buildBasePayload(init);
    assignIfDefined(payload, 'label', init.label);
    if (init.checked !== undefined) {
      payload.checked = normalizeBool(init.checked, 'checked');
    }
    await this.createWidget(id, 'checkbox', payload);
    return new RemoteCheckbox(id, this);
  }

  async label(id: string, init: LabelInit = {}): Promise<RemoteLabel> {
    const payload = buildBasePayload(init);
    assignIfDefined(payload, 'text', init.text);
    if (init.color !== undefined) {
      payload.color = normalizeColor(init.color, 'color');
    }
    await this.createWidget(id, 'label', payload);
    return new RemoteLabel(id, this);
  }

  async textbox(id: string, init: TextBoxInit = {}): Promise<RemoteTextBox> {
    const payload = buildBasePayload(init);
    assignIfDefined(payload, 'text', init.text);
    assignIfDefined(payload, 'placeholder', normalizeOptionalString(init.placeholder, 'placeholder'));
    await this.createWidget(id, 'textbox', payload);
    return new RemoteTextBox(id, this);
  }

  async dropdown(id: string, init: DropdownInit = {}): Promise<RemoteDropdown> {
    const payload = buildBasePayload(init);
    if (init.options !== undefined) {
      payload.options = Array.from(init.options, (option, index) => {
        if (typeof option !== 'string') {
          throw new Error(`options[${index}] must be a string`);
        }
        return option;
      });
    }
    assignIfDefined(payload, 'placeholder', normalizeOptionalString(init.placeholder ?? undefined, 'placeholder'));
    if (init.maxVisibleItems !== undefined) {
      payload.max_visible_items = normalizeInteger(init.maxVisibleItems, 'maxVisibleItems');
    }
    if (init.optionHeight !== undefined) {
      payload.option_height = ensureFinite(init.optionHeight, 'optionHeight');
    }
    if (init.selectedIndex !== undefined) {
      payload.selected_index = normalizeInteger(init.selectedIndex, 'selectedIndex');
    }
    await this.createWidget(id, 'dropdown', payload);
    return new RemoteDropdown(id, this);
  }

  async panel(id: string, init: PanelInit = {}): Promise<RemotePanel> {
    const payload = buildBasePayload(init);
    assignIfDefined(payload, 'title', init.title);
    await this.createWidget(id, 'panel', payload);
    return new RemotePanel(id, this);
  }

  async horizontalLayout(id: string, init: LayoutInit = {}): Promise<RemoteHorizontalLayout> {
    const payload: Record<string, unknown> = {};
    if (init.position !== undefined) {
      payload.position = normalizeVec2(init.position, 'position');
    }
    await this.createWidget(id, 'horizontal_layout', payload);
    return new RemoteHorizontalLayout(id, this);
  }

  async verticalLayout(id: string, init: LayoutInit = {}): Promise<RemoteVerticalLayout> {
    const payload: Record<string, unknown> = {};
    if (init.position !== undefined) {
      payload.position = normalizeVec2(init.position, 'position');
    }
    await this.createWidget(id, 'vertical_layout', payload);
    return new RemoteVerticalLayout(id, this);
  }

  async sendWidgetCommand(id: string, method: string, params?: unknown): Promise<void> {
    await this.send({ id, method, params: params ?? {} });
  }

  private async createWidget(id: string, kind: WidgetKind, params: Record<string, unknown>): Promise<void> {
    await this.send({
      id,
      method: 'create',
      params: {
        kind,
        ...params,
      },
    });
  }

  private async send(command: RemoteCommand): Promise<void> {
    const payload = {
      id: command.id,
      method: command.method,
      params: command.params ?? {},
    };
    await this.sender(JSON.stringify(payload));
  }
}

export abstract class RemoteWidgetBase {
  protected readonly commands: WidgetCommandBuilder;

  constructor(public readonly id: string, protected readonly client: RemoteUiClient) {
    this.commands = new WidgetCommandBuilder(client, id);
  }

  /**
   * Attaches this widget to a parent container.
   */
  async attachTo(parent: string | RemoteWidgetBase, offset?: Vec2Like): Promise<this> {
    await this.client.attachChild(parent, this, offset);
    return this;
  }

  /**
   * Destroys the widget, mirroring the host's `destroy` command.
   */
  async destroy(): Promise<void> {
    await this.client.destroy(this);
  }

  /**
   * Registers an event handler for this widget. The callback will be invoked when the event occurs.
   * Supports both synchronous and asynchronous callbacks.
   * @param eventType The type of event to listen for (e.g., 'click', 'change', 'toggle')
   * @param callback The function to call when the event occurs (can be async)
   * @returns This widget instance for method chaining
   */
  on<T = any>(eventType: string, callback: (event: T) => void | Promise<void>): this {
    if (typeof __registerUiCallback !== 'function') {
      throw new Error('__registerUiCallback is not available. Event handling may not be supported in this runtime.');
    }
    __registerUiCallback(this.id, eventType, callback);
    return this;
  }

  /**
   * Unregisters an event handler for this widget.
   * @param eventType The type of event to stop listening for
   * @returns This widget instance for method chaining
   */
  off(eventType: string): this {
    if (typeof __unregisterUiCallback !== 'function') {
      throw new Error('__unregisterUiCallback is not available. Event handling may not be supported in this runtime.');
    }
    __unregisterUiCallback(this.id, eventType);
    return this;
  }

  protected async send(method: string, params?: unknown): Promise<void> {
    await this.commands.send(method, params);
  }
}

export class RemoteButton extends RemoteWidgetBase {
  /**
   * Registers a click event handler for this button.
   * Supports both synchronous and asynchronous callbacks.
   */
  on<T = any>(eventType: string, callback: (event: T) => void | Promise<void>): this;
  on(eventType: 'click', callback: (event: ButtonClickEvent) => void | Promise<void>): this;
  on(eventType: string, callback: (event: any) => void | Promise<void>): this {
    return super.on(eventType, callback);
  }

  async setPosition(position: Vec2Like): Promise<this> {
    await this.send('set_position', normalizeVec2(position, 'position'));
    return this;
  }

  async setSize(size: SizeLike): Promise<this> {
    await this.send('set_size', normalizeSize(size, 'size'));
    return this;
  }

  async setLabel(label: string): Promise<this> {
    await this.send('set_label', { text: normalizeString(label, 'label') });
    return this;
  }

  async setColors(colors: ButtonColors): Promise<this> {
    await this.send('set_colors', {
      normal: normalizeColor(colors.normal, 'colors.normal'),
      hover: normalizeColor(colors.hover, 'colors.hover'),
      pressed: normalizeColor(colors.pressed, 'colors.pressed'),
    });
    return this;
  }

  async setTextColor(color: ColorLike): Promise<this> {
    await this.send('set_text_color', normalizeColor(color, 'textColor'));
    return this;
  }

  async setBorderColor(color: ColorLike): Promise<this> {
    await this.send('set_border_color', normalizeColor(color, 'borderColor'));
    return this;
  }

  async setHovered(state: boolean): Promise<this> {
    await this.send('set_hovered', { value: normalizeBool(state, 'hovered') });
    return this;
  }

  async setPressed(state: boolean): Promise<this> {
    await this.send('set_pressed', { value: normalizeBool(state, 'pressed') });
    return this;
  }
}

export class RemoteCheckbox extends RemoteWidgetBase {
  /**
   * Registers a toggle event handler for this checkbox.
   * Supports both synchronous and asynchronous callbacks.
   */
  on<T = any>(eventType: string, callback: (event: T) => void | Promise<void>): this;
  on(eventType: 'toggle', callback: (event: CheckboxToggleEvent) => void | Promise<void>): this;
  on(eventType: string, callback: (event: any) => void | Promise<void>): this {
    return super.on(eventType, callback);
  }

  async setPosition(position: Vec2Like): Promise<this> {
    await this.send('set_position', normalizeVec2(position, 'position'));
    return this;
  }

  async setSize(size: SizeLike): Promise<this> {
    await this.send('set_size', normalizeSize(size, 'size'));
    return this;
  }

  async setChecked(checked: boolean): Promise<this> {
    await this.send('set_checked', { value: normalizeBool(checked, 'checked') });
    return this;
  }

  async setLabel(label: string): Promise<this> {
    await this.send('set_label', { text: normalizeString(label, 'label') });
    return this;
  }
}

export class RemoteLabel extends RemoteWidgetBase {
  async setPosition(position: Vec2Like): Promise<this> {
    await this.send('set_position', normalizeVec2(position, 'position'));
    return this;
  }

  async setSize(size: SizeLike): Promise<this> {
    await this.send('set_size', normalizeSize(size, 'size'));
    return this;
  }

  async setText(text: string): Promise<this> {
    await this.send('set_text', { text: normalizeString(text, 'text') });
    return this;
  }

  async setColor(color: ColorLike): Promise<this> {
    await this.send('set_color', normalizeColor(color, 'color'));
    return this;
  }

  async setPaletteColor(slot: PaletteSlot): Promise<this> {
    await this.send('set_palette_color', { slot });
    return this;
  }
}

export class RemoteTextBox extends RemoteWidgetBase {
  /**
   * Registers event handlers for this textbox.
   * Supports both synchronous and asynchronous callbacks.
   */
  on<T = any>(eventType: string, callback: (event: T) => void | Promise<void>): this;
  on(eventType: 'change', callback: (event: TextChangeEvent) => void | Promise<void>): this;
  on(eventType: 'focus', callback: (event: TextBoxFocusEvent) => void | Promise<void>): this;
  on(eventType: string, callback: (event: any) => void | Promise<void>): this {
    return super.on(eventType, callback);
  }

  async setPosition(position: Vec2Like): Promise<this> {
    await this.send('set_position', normalizeVec2(position, 'position'));
    return this;
  }

  async setSize(size: SizeLike): Promise<this> {
    await this.send('set_size', normalizeSize(size, 'size'));
    return this;
  }

  async setText(text: string): Promise<this> {
    await this.send('set_text', { text: normalizeString(text, 'text') });
    return this;
  }

  async setFocused(focused: boolean): Promise<this> {
    await this.send('set_focused', { value: normalizeBool(focused, 'focused') });
    return this;
  }

  async setPlaceholder(placeholder: string): Promise<this> {
    await this.send('set_placeholder', { text: normalizeString(placeholder, 'placeholder') });
    return this;
  }
}

export class RemoteDropdown extends RemoteWidgetBase {
  /**
   * Registers a change event handler for this dropdown.
   * Supports both synchronous and asynchronous callbacks.
   */
  on<T = any>(eventType: string, callback: (event: T) => void | Promise<void>): this;
  on(eventType: 'change', callback: (event: DropdownChangeEvent) => void | Promise<void>): this;
  on(eventType: string, callback: (event: any) => void | Promise<void>): this {
    return super.on(eventType, callback);
  }

  async setPosition(position: Vec2Like): Promise<this> {
    await this.send('set_position', normalizeVec2(position, 'position'));
    return this;
  }

  async setSize(size: SizeLike): Promise<this> {
    await this.send('set_size', normalizeSize(size, 'size'));
    return this;
  }

  async setSelectedIndex(index: number): Promise<this> {
    await this.send('set_selected_index', { index: normalizeInteger(index, 'index') });
    return this;
  }

  async setOptions(options: Iterable<string>): Promise<this> {
    await this.send('set_options', {
      options: Array.from(options, (option, idx) => {
        if (typeof option !== 'string') {
          throw new Error(`options[${idx}] must be a string`);
        }
        return option;
      }),
    });
    return this;
  }

  async setPlaceholder(placeholder: string | null): Promise<this> {
    const text = normalizeOptionalString(placeholder, 'placeholder');
    const params = text === undefined ? {} : { text };
    await this.send('set_placeholder', params);
    return this;
  }

  async setMaxVisibleItems(count: number): Promise<this> {
    await this.send('set_max_visible_items', { count: normalizeInteger(count, 'count') });
    return this;
  }

  async setOptionHeight(height: number): Promise<this> {
    await this.send('set_option_height', { value: ensureFinite(height, 'height') });
    return this;
  }

  async setOpen(open: boolean): Promise<this> {
    await this.send('set_open', { value: normalizeBool(open, 'open') });
    return this;
  }
}

export class RemotePanel extends RemoteWidgetBase {
  /**
   * Registers event handlers for this panel.
   * Supports both synchronous and asynchronous callbacks.
   */
  on<T = any>(eventType: string, callback: (event: T) => void | Promise<void>): this;
  on(eventType: 'dragstart', callback: (event: PanelDragStartEvent) => void | Promise<void>): this;
  on(eventType: 'drag', callback: (event: PanelDragEvent) => void | Promise<void>): this;
  on(eventType: 'dragend', callback: (event: PanelDragEndEvent) => void | Promise<void>): this;
  on(eventType: 'togglechange', callback: (event: PanelToggleChangeEvent) => void | Promise<void>): this;
  on(eventType: string, callback: (event: any) => void | Promise<void>): this {
    return super.on(eventType, callback);
  }

  async setPosition(position: Vec2Like): Promise<this> {
    await this.send('set_position', normalizeVec2(position, 'position'));
    return this;
  }

  async setSize(size: SizeLike): Promise<this> {
    await this.send('set_size', normalizeSize(size, 'size'));
    return this;
  }

  async setTitle(title: string): Promise<this> {
    await this.send('set_title', { text: normalizeString(title, 'title') });
    return this;
  }

  async setColors(colors: PanelColors): Promise<this> {
    await this.send('set_colors', {
      background: normalizeColor(colors.background, 'colors.background'),
      title_bar: normalizeColor(colors.titleBar, 'colors.titleBar'),
    });
    return this;
  }

  async setBorderColor(color: ColorLike): Promise<this> {
    await this.send('set_border_color', normalizeColor(color, 'borderColor'));
    return this;
  }

  async setPadding(padding: PaddingLike): Promise<this> {
    await this.send('set_padding', normalizeVec2(padding, 'padding'));
    return this;
  }

  async clearChildren(): Promise<this> {
    await this.send('clear_children', {});
    return this;
  }

  async addChild(child: string | RemoteWidgetBase, offset?: Vec2Like): Promise<this> {
    await this.client.attachChild(this, child, offset);
    return this;
  }
}

abstract class RemoteLayoutBase extends RemoteWidgetBase {
  async setPosition(position: Vec2Like): Promise<this> {
    await this.send('set_position', normalizeVec2(position, 'position'));
    return this;
  }

  async setSpacing(spacing: number): Promise<this> {
    await this.send('set_spacing', { value: ensureFinite(spacing, 'spacing') });
    return this;
  }

  async setPadding(padding: PaddingLike): Promise<this> {
    await this.send('set_padding', normalizeVec2(padding, 'padding'));
    return this;
  }

  async setCrossAlignment(alignment: CrossAlignment): Promise<this> {
    const normalized =
      alignment === 'centre' ? 'center' : alignment;
    if (!['start', 'center', 'end'].includes(normalized)) {
      throw new Error(`alignment must be 'start', 'center', or 'end'`);
    }
    await this.send('set_cross_alignment', { alignment: normalized });
    return this;
  }

  async recomputeLayout(): Promise<this> {
    await this.send('recompute_layout', {});
    return this;
  }

  async addChild(child: string | RemoteWidgetBase): Promise<this> {
    await this.client.attachChild(this, child);
    return this;
  }
}

export class RemoteHorizontalLayout extends RemoteLayoutBase {}

export class RemoteVerticalLayout extends RemoteLayoutBase {}

export { RemoteUiClient as default };
