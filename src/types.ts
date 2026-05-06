export interface Component {
  name: string;
  description: string;
  requires_input: boolean;
  produces_output: boolean;
  is_builtin: boolean;
  default_config: any;
  instructions?: string;
  ui_schema?: any;
  group?: string;
}

export interface Node {
  id: string;
  component_name: string;
  pass_through?: boolean;
  delay_before_ms?: number | null;
  config?: any | null;
  
  // 纯 UI 状态，将不会被传递给后端（如果在保存前清理的话）
  // 更好的做法是在保存时过滤掉这些
  _capturing?: boolean;
  _countdown?: number;
}

export interface Flow {
  id: string;
  display_name: string;
  startup_delay_ms: number;
  description?: string | null;
  group?: string | null;
  shortcut?: string | null;
  nodes?: Node[] | null;
  cwd?: string | null;
}
