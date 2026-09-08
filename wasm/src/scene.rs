use ochub_ui::components::{self, BadgeTone, ButtonSize, ButtonTone};
use ochub_ui::gpui::{
    Context, FontWeight, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
    prelude::*, px,
};
use ochub_ui::icons::{self, IconName};
use ochub_ui::screens::{
    common as common_screen, gateway as gateway_screen, provider_editor as editor_screen,
    providers as provider_screens, settings as settings_screen, shell, themes as theme_screen,
    tools as tool_screens, usage as usage_screen,
};
use ochub_ui::{layout, theme};

#[derive(Clone)]
pub struct SceneConfig {
    pub scene: String,
    pub locale: String,
    pub app: String,
    pub current_name: String,
    pub connection_name: String,
    pub endpoint: String,
}

impl SceneConfig {
    pub fn from_browser() -> Self {
        let search = web_sys::window()
            .and_then(|window| window.location().search().ok())
            .unwrap_or_default();
        let pairs = search
            .trim_start_matches('?')
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .map(|(key, value)| (decode(key), decode(value)))
            .collect::<Vec<_>>();
        let get = |name: &str, fallback: &str| {
            pairs
                .iter()
                .find_map(|(key, value)| (key == name).then(|| value.clone()))
                .unwrap_or_else(|| fallback.to_string())
        };
        let app = get("app", "Claude Code");
        let default_endpoint = match app.as_str() {
            "Codex" => "http://127.0.0.1:4180/connection/rd-example/backend-api/codex",
            "Grok Build" => "https://api.x.ai/v1",
            _ => "https://api.example.com/claude",
        };
        Self {
            scene: get("scene", "connections"),
            locale: get("locale", "en"),
            app,
            current_name: get("currentName", "Official account"),
            connection_name: get("connectionName", "Example API"),
            endpoint: get("endpoint", default_endpoint),
        }
    }
}

fn decode(value: &str) -> String {
    js_sys::decode_uri_component(&value.replace('+', " "))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| value.to_string())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Connections,
    Editor,
    Providers,
    Mcp,
    Skills,
    Sessions,
    Usage,
    Settings,
    Theme,
    Sync,
    Welcome,
}

#[derive(Clone, Copy)]
struct Copy {
    workspace: &'static str,
    local: &'static str,
    apps: &'static str,
    tools: &'static str,
    network: &'static str,
    system: &'static str,
    providers: &'static str,
    mcp: &'static str,
    skills: &'static str,
    sessions: &'static str,
    usage: &'static str,
    settings: &'static str,
    themes: &'static str,
    advanced_tools: &'static str,
    proxy: &'static str,
    remote: &'static str,
    about: &'static str,
    add: &'static str,
    save: &'static str,
    edit: &'static str,
    copy: &'static str,
    delete: &'static str,
    switch: &'static str,
    current: &'static str,
    direct: &'static str,
    name: &'static str,
    base_url: &'static str,
    api_key: &'static str,
    test: &'static str,
    models: &'static str,
    files: &'static str,
    enabled: &'static str,
}

impl Copy {
    fn for_locale(locale: &str) -> Self {
        match locale.split(['-', '_']).next().unwrap_or("en") {
            "zh" => Self {
                workspace: "工作区",
                local: "这台 Mac",
                apps: "应用",
                tools: "工具",
                network: "网络",
                system: "系统",
                providers: "模型供应商",
                mcp: "MCP 服务器",
                skills: "技能",
                sessions: "会话",
                usage: "用量",
                settings: "设置",
                themes: "主题",
                advanced_tools: "高级工具",
                proxy: "网络代理",
                remote: "远程节点",
                about: "关于",
                add: "添加",
                save: "保存",
                edit: "编辑",
                copy: "复制",
                delete: "删除",
                switch: "切换",
                current: "当前",
                direct: "直接连接",
                name: "名称",
                base_url: "Base URL",
                api_key: "API Key",
                test: "测试 URL",
                models: "拉取模型",
                files: "将写入的文件",
                enabled: "已启用",
            },
            "ja" => Self {
                workspace: "ワークスペース",
                local: "この Mac",
                apps: "アプリ",
                tools: "ツール",
                network: "ネットワーク",
                system: "システム",
                providers: "モデルプロバイダー",
                mcp: "MCP サーバー",
                skills: "スキル",
                sessions: "セッション",
                usage: "使用量",
                settings: "設定",
                themes: "テーマ",
                advanced_tools: "高度なツール",
                proxy: "ネットワークプロキシ",
                remote: "リモートノード",
                about: "このアプリについて",
                add: "追加",
                save: "保存",
                edit: "編集",
                copy: "コピー",
                delete: "削除",
                switch: "切り替え",
                current: "現在",
                direct: "直接接続",
                name: "名前",
                base_url: "Base URL",
                api_key: "API キー",
                test: "URL をテスト",
                models: "モデルを取得",
                files: "書き込むファイル",
                enabled: "有効",
            },
            _ => Self {
                workspace: "Workspace",
                local: "This Mac",
                apps: "Apps",
                tools: "Tools",
                network: "Network",
                system: "System",
                providers: "Model providers",
                mcp: "MCP servers",
                skills: "Skills",
                sessions: "Sessions",
                usage: "Usage",
                settings: "Settings",
                themes: "Themes",
                advanced_tools: "Advanced tools",
                proxy: "Network proxy",
                remote: "Remote nodes",
                about: "About",
                add: "Add",
                save: "Save",
                edit: "Edit",
                copy: "Copy",
                delete: "Delete",
                switch: "Switch",
                current: "Current",
                direct: "Direct connection",
                name: "Name",
                base_url: "Base URL",
                api_key: "API key",
                test: "Test URL",
                models: "Fetch models",
                files: "Files to be written",
                enabled: "Enabled",
            },
        }
    }
}

pub struct SceneView {
    config: SceneConfig,
}

impl SceneView {
    fn codex_guide(&self) -> ochub_ui::gpui::Div {
        let tr = |en, zh, ja| match self.config.locale.as_str() {
            "zh" => zh,
            "ja" => ja,
            _ => en,
        };
        let (title, rows): (&str, Vec<(&str, &str)>) = match self.config.scene.as_str() {
            "codex-guide-1" => (
                tr("Model provider", "模型供应商", "モデルプロバイダー"),
                vec![
                    (tr("Name", "名称", "名前"), "Example Responses"),
                    ("URL", "https://api.example.com/openai/v1/responses"),
                    ("API Key", "••••••••"),
                    (tr("Protocol", "协议", "プロトコル"), "Responses"),
                ],
            ),
            "codex-guide-2" => (
                "Codex",
                vec![
                    (
                        tr("Model source", "模型来源", "モデルの接続先"),
                        tr("Model provider", "模型供应商", "モデルプロバイダー"),
                    ),
                    (
                        tr("Provider", "供应商", "プロバイダー"),
                        "Example Responses",
                    ),
                    (tr("Model", "模型", "モデル"), "gpt-5-codex"),
                ],
            ),
            "codex-guide-3" => (
                "Codex / ChatGPT",
                vec![
                    (tr("Account", "账号", "アカウント"), "ChatGPT"),
                    (tr("Credentials", "登录凭据", "ログイン情報"), "auth.json"),
                    (
                        tr("Virtual login", "虚拟登录", "仮想ログイン"),
                        tr("Off", "关闭", "無効"),
                    ),
                ],
            ),
            "codex-guide-4" => (
                tr("Context management", "上下文管理", "コンテキスト管理"),
                vec![
                    (
                        tr("Context management", "上下文管理", "コンテキスト管理"),
                        tr("On", "开启", "有効"),
                    ),
                    ("enabled", "true"),
                    ("use_history_notes_extension", "true"),
                ],
            ),
            "codex-guide-5" => (
                tr("Review and activate", "检查并启用", "確認して有効化"),
                vec![
                    (tr("Target", "目标应用", "対象アプリ"), "Codex"),
                    (
                        tr("Configuration", "配置文件", "設定ファイル"),
                        "~/.codex/config.toml",
                    ),
                    (
                        tr("Action in OcHub", "在 OcHub 中操作", "OcHub での操作"),
                        tr("Save and activate", "保存并启用", "保存して有効化"),
                    ),
                ],
            ),
            _ => (
                tr("Verify actual results", "验证实际结果", "実際の結果を確認"),
                vec![
                    (tr("Inference", "模型请求", "推論"), "/responses"),
                    ("History", "alpha/history/v2/*"),
                    ("Notes", "alpha/notes/v2/*"),
                    (
                        tr("Result", "结果", "結果"),
                        tr("Check service logs", "检查分发服务日志", "配信ログを確認"),
                    ),
                ],
            ),
        };
        // Read-only, focused illustrations: no fake save buttons or input events.
        let mut body = div().flex().flex_col().gap_3().p_4().w_full();
        body = body.child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        );
        for (index, (label, value)) in rows.into_iter().enumerate() {
            body = body.child(
                div()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(if index == 0 {
                        theme::accent()
                    } else {
                        theme::border()
                    })
                    .bg(if index == 0 {
                        theme::accent_soft()
                    } else {
                        theme::inset()
                    })
                    .child(div().text_xs().text_color(theme::muted()).child(label))
                    .child(div().text_sm().child(value)),
            );
        }
        body
    }

    pub fn new(config: SceneConfig) -> Self {
        Self { config }
    }

    fn is(&self, names: &[&str]) -> bool {
        names.contains(&self.config.scene.as_str())
    }

    fn page(&self) -> Page {
        let scene = self.config.scene.as_str();
        if matches!(
            scene,
            "editor" | "source" | "identity" | "fields" | "tests" | "preview" | "save" | "model"
        ) {
            Page::Editor
        } else if matches!(
            scene,
            "providers"
                | "provider-nav"
                | "provider-add"
                | "provider-fields"
                | "detect"
                | "provider-model"
                | "model-provider-card"
        ) {
            Page::Providers
        } else if scene.starts_with("mcp-") {
            Page::Mcp
        } else if scene.starts_with("skill") {
            Page::Skills
        } else if scene.starts_with("session") {
            Page::Sessions
        } else if matches!(
            scene,
            "usage-nav"
                | "usage-summary"
                | "usage-filter"
                | "request-detail"
                | "pricing"
                | "pricing-save"
        ) {
            Page::Usage
        } else if matches!(scene, "theme-nav" | "theme-mode" | "theme-card") {
            Page::Theme
        } else if scene.starts_with("sync-") || scene.starts_with("backup-") {
            Page::Sync
        } else if matches!(scene, "welcome" | "import") {
            Page::Welcome
        } else if matches!(scene, "settings-nav" | "apps-settings") {
            Page::Settings
        } else {
            Page::Connections
        }
    }

    fn targeted(&self, names: &[&str], child: impl IntoElement) -> ochub_ui::gpui::Div {
        div()
            .rounded_lg()
            .when(self.is(names), |node| {
                node.p(px(3.))
                    .bg(theme::accent_soft())
                    .border_2()
                    .border_color(theme::accent())
            })
            .child(child)
    }

    fn nav_item(
        &self,
        id: &'static str,
        label: &str,
        icon: IconName,
        selected: bool,
        target: &[&str],
        appearance: ochub_ui::gpui::WindowAppearance,
    ) -> impl IntoElement + use<> {
        shell::nav_item(id, label.to_string(), icon, selected, appearance)
            .when(self.is(target), |row| {
                row.border_2().border_color(theme::accent())
            })
    }

    fn sidebar(
        &self,
        page: Page,
        copy: Copy,
        appearance: ochub_ui::gpui::WindowAppearance,
    ) -> impl IntoElement + use<> {
        let app_selected = matches!(page, Page::Connections | Page::Editor);
        let app_item = |id, label: &str, icon, accent, selected, target: &[&str]| {
            shell::app_item(id, label.to_string(), icon, accent, selected, appearance)
                .when(self.is(target), |row| {
                    row.border_2().border_color(theme::accent())
                })
                .into_any_element()
        };
        let navigation = shell::navigation([
            shell::group_label(copy.workspace, appearance).into_any_element(),
            div()
                .w_full()
                .px_2()
                .child(components::select_dropdown_sidebar(
                    "workspace-scope",
                    &[copy.local],
                    0,
                    false,
                    appearance,
                    |_event, _window, _cx| {},
                ))
                .into_any_element(),
            shell::group_label(copy.apps, appearance).into_any_element(),
            shell::item_group([
                app_item(
                    "app-claude-code",
                    &self.config.app,
                    match self.config.app.as_str() {
                        "Grok Build" => IconName::AgentGrokBuild,
                        "Codex" => IconName::AgentCodex,
                        _ => IconName::AgentClaudeCode,
                    },
                    0xd97757,
                    app_selected,
                    &["sidebar-app"],
                ),
                app_item(
                    "app-claude-desktop",
                    "Claude Desktop",
                    IconName::AgentClaude,
                    0xbd5d3a,
                    false,
                    &[],
                ),
                app_item(
                    "app-codex",
                    "Codex",
                    IconName::AgentCodex,
                    0x0d0d0d,
                    false,
                    &[],
                ),
                app_item(
                    "app-opencode",
                    "OpenCode",
                    IconName::AgentOpenCode,
                    0x2b2b33,
                    false,
                    &[],
                ),
            ])
            .into_any_element(),
            shell::group_label(copy.tools, appearance).into_any_element(),
            shell::item_group([
                self.nav_item(
                    "nav-mcp",
                    copy.mcp,
                    IconName::Blocks,
                    page == Page::Mcp,
                    &["mcp-nav"],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-skills",
                    copy.skills,
                    IconName::Pencil,
                    page == Page::Skills,
                    &["skills-nav"],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-usage",
                    copy.usage,
                    IconName::Chart,
                    page == Page::Usage,
                    &["usage-nav"],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-sessions",
                    copy.sessions,
                    IconName::Message,
                    page == Page::Sessions,
                    &["sessions-nav"],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-tools",
                    copy.advanced_tools,
                    IconName::Tools,
                    false,
                    &[],
                    appearance,
                )
                .into_any_element(),
            ])
            .into_any_element(),
            shell::group_label(copy.network, appearance).into_any_element(),
            shell::item_group([
                self.nav_item(
                    "nav-gateway",
                    copy.providers,
                    IconName::Cloud,
                    page == Page::Providers,
                    &["provider-nav"],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-network",
                    copy.proxy,
                    IconName::Globe,
                    false,
                    &[],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-remote",
                    copy.remote,
                    IconName::Desktop,
                    false,
                    &[],
                    appearance,
                )
                .into_any_element(),
            ])
            .into_any_element(),
            shell::group_label(copy.system, appearance).into_any_element(),
            shell::item_group([
                self.nav_item(
                    "nav-themes",
                    copy.themes,
                    IconName::Palette,
                    page == Page::Theme,
                    &["theme-nav"],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-settings",
                    copy.settings,
                    IconName::Settings,
                    matches!(page, Page::Settings | Page::Sync),
                    &["settings-nav"],
                    appearance,
                )
                .into_any_element(),
                self.nav_item(
                    "nav-about",
                    copy.about,
                    IconName::Desktop,
                    false,
                    &[],
                    appearance,
                )
                .into_any_element(),
            ])
            .into_any_element(),
        ]);

        shell::sidebar(
            appearance,
            div().w_full().h(px(44.)).flex_none(),
            navigation,
        )
    }

    fn connection_page(&self, copy: Copy) -> ochub_ui::gpui::Div {
        let rows: [(&str, &str); 3] = [
            (
                self.config.connection_name.as_str(),
                self.config.endpoint.as_str(),
            ),
            ("Team gateway", "https://gateway.example.com"),
            ("Development", "http://127.0.0.1:8080"),
        ];
        let mut list = div().flex().flex_col().gap_3().w_full();
        for (index, (name, endpoint)) in rows.into_iter().enumerate() {
            let row = provider_screens::provider_card(provider_screens::ProviderCard {
                id: SharedString::from(format!("provider-card-{index}")),
                name: SharedString::from(name.to_string()),
                endpoint: SharedString::from(endpoint.to_string()),
                icon: IconName::AgentClaudeCode,
                is_current: false,
                is_drag_source: false,
                drag_handle: Some(
                    div()
                        .id(format!("drag-provider-{index}"))
                        .role(ochub_ui::gpui::Role::Button)
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(32.))
                        .flex_none()
                        .child(icons::icon(IconName::DragHandle, theme::muted(), 16.))
                        .into_any_element(),
                ),
                quota: None,
                current_label: SharedString::from(copy.current),
                actions: vec![
                    components::action_button(format!("edit-{index}"), copy.edit, false)
                        .into_any_element(),
                    components::action_button(format!("copy-{index}"), copy.copy, false)
                        .into_any_element(),
                    components::action_button_tone(
                        format!("delete-{index}"),
                        copy.delete,
                        ButtonTone::Danger,
                    )
                    .into_any_element(),
                    components::action_button(format!("switch-{index}"), copy.switch, true)
                        .into_any_element(),
                ],
            });
            list = list.child(self.targeted(
                &[
                    "connection-card",
                    "connection-add",
                    "connection-switch",
                    "connection-copy",
                    "connection-remove",
                ],
                row,
            ));
        }
        let subtitle = match self.config.locale.split(['-', '_']).next() {
            Some("zh") => "3 个连接",
            Some("ja") => "3 件の接続",
            _ => "3 connections",
        };
        let actions = div()
            .flex()
            .flex_row()
            .gap_2()
            .child(self.targeted(
                &["toolbar-add"],
                components::icon_button("add", copy.add, IconName::Add, true),
            ))
            .child(components::icon_button(
                "app-settings",
                copy.settings,
                IconName::Settings,
                false,
            ));
        let hero = provider_screens::active_provider_hero(provider_screens::ActiveProviderHero {
            icon: IconName::AgentClaudeCode,
            accent: 0xd97757,
            is_gateway: false,
            current_label: SharedString::from(copy.current),
            direct_label: SharedString::from(copy.direct),
            name: Some(SharedString::from(self.config.current_name.clone())),
            endpoint: Some(SharedString::from("—")),
            quota: None,
            empty_title: SharedString::from("—"),
            empty_hint: SharedString::from(""),
            actions: vec![
                components::action_button("current-edit", copy.edit, false).into_any_element(),
            ],
        });
        provider_screens::provider_list_page(
            self.config.app.clone(),
            subtitle,
            actions,
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p_6()
                .items_center()
                .child(
                    layout::wide_column()
                        .child(hero)
                        .child(layout::section_header(copy.direct, None))
                        .child(list),
                ),
        )
    }

    fn editor_page(&self, copy: Copy, window_width: f32) -> ochub_ui::gpui::Div {
        let (title, subtitle, source_label, website, category, notes, cancel, copy_to_app) =
            match self.config.locale.split(['-', '_']).next() {
                Some("zh") => (
                    "编辑连接",
                    "目标应用：",
                    "来源",
                    "网站 URL",
                    "分类",
                    "备注",
                    "取消",
                    "复制到应用",
                ),
                Some("ja") => (
                    "接続を編集",
                    "対象アプリ：",
                    "ソース",
                    "Web サイト URL",
                    "カテゴリ",
                    "メモ",
                    "キャンセル",
                    "アプリへコピー",
                ),
                _ => (
                    "Edit connection",
                    "Target app: ",
                    "Source",
                    "Website URL",
                    "Category",
                    "Notes",
                    "Cancel",
                    "Copy to app",
                ),
            };
        let control = |value: String, multiline: bool| {
            div()
                .w_full()
                .min_h(if multiline { px(72.) } else { px(38.) })
                .rounded_md()
                .bg(theme::inset())
                .border_1()
                .border_color(theme::border())
                .px_3()
                .py_2()
                .text_sm()
                .child(value)
        };
        let form = div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full()
            .min_w_0()
            .child(self.targeted(
                &["source"],
                components::field(
                    source_label,
                    false,
                    None,
                    components::segmented_readonly(
                        "provider-source",
                        &[copy.direct, copy.providers],
                        usize::from(self.config.app == "Codex"),
                    ),
                ),
            ))
            .when(self.config.app == "Codex", |form| {
                let (account, context, enabled) = match self.config.locale.as_str() {
                    "zh" => ("账号", "上下文管理", "开启"),
                    "ja" => ("アカウント", "コンテキスト管理", "有効"),
                    _ => ("Account", "Context management", "On"),
                };
                form.child(components::field(
                    account,
                    false,
                    None,
                    control("ChatGPT".into(), false),
                ))
                .child(components::field(
                    context,
                    false,
                    None,
                    control(enabled.into(), false),
                ))
            })
            .child(self.targeted(
                &["identity"],
                components::field(
                    copy.name,
                    true,
                    None,
                    control(self.config.current_name.clone(), false),
                ),
            ))
            .when(self.config.app != "Codex", |form| {
                form.child(components::field(
                    website,
                    false,
                    None,
                    control(
                        match self.config.app.as_str() {
                            "Codex" => "https://chatgpt.com/codex",
                            "Grok Build" => "https://x.ai",
                            _ => "https://www.anthropic.com/claude-code",
                        }
                        .into(),
                        false,
                    ),
                ))
                .child(components::field(
                    category,
                    false,
                    None,
                    control("official".into(), false),
                ))
                .child(components::field(
                    notes,
                    false,
                    None,
                    control("Personal account".into(), true),
                ))
            })
            .when(self.config.app != "Codex", |form| {
                form.child(layout::section_header(
                    match self.config.locale.as_str() {
                        "zh" => "端点与鉴权",
                        "ja" => "エンドポイントと認証",
                        _ => "Endpoint and authentication",
                    },
                    None,
                ))
                .child(self.targeted(
                    &["fields"],
                    components::field(
                        copy.base_url,
                        true,
                        None,
                        control(self.config.endpoint.clone(), false),
                    ),
                ))
                .child(components::field(
                    copy.api_key,
                    true,
                    None,
                    control("••••••••••••••••".into(), false),
                ))
            })
            .child(self.targeted(
                &["model"],
                components::field(
                    match self.config.locale.as_str() {
                        "zh" => "默认模型",
                        "ja" => "デフォルトモデル",
                        _ => "Default model",
                    },
                    false,
                    None,
                    control(sample_model(&self.config.app).into(), false),
                ),
            ))
            .child(
                self.targeted(
                    &["tests"],
                    div()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .child(components::button(
                            "test",
                            copy.test,
                            ButtonTone::Neutral,
                            ButtonSize::Sm,
                        ))
                        .child(components::button(
                            "models",
                            copy.models,
                            ButtonTone::Neutral,
                            ButtonSize::Sm,
                        )),
                ),
            );
        let form_scroll = div()
            .id("docs-editor-form-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_y_scroll()
            .pr_2()
            .child(form)
            .into_any_element();
        let compact_layout = editor_screen::is_compact(window_width);
        let preview = if compact_layout {
            Some(
                self.targeted(
                    &["preview"],
                    components::card()
                        .p_3()
                        .child(div().child(copy.files))
                        .child(div().font_family("Lilex").text_xs().child(
                            match self.config.app.as_str() {
                                "Codex" => "~/.codex/config.toml",
                                "Grok Build" => "~/.grok/",
                                _ => "~/.claude/settings.json",
                            },
                        )),
                )
                .into_any_element(),
            )
        } else {
            Some(
                components::card()
                    .p_0()
                    .w(px(editor_screen::preview_width(window_width)))
                    .h_full()
                    .flex_none()
                    .child(
                        div()
                            .px_4()
                            .py_3()
                            .border_b_1()
                            .border_color(theme::border())
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(copy.files),
                    )
                    .child(
                        div()
                            .p_4()
                            .font_family("Lilex")
                            .text_xs()
                            .child(match self.config.app.as_str() {
                                "Codex" => "[features.token_budget]\nenabled = true\nuse_history_notes_extension = true",
                                "Grok Build" => "API backend: OpenAI Responses\nModel: grok-4.5",
                                _ => "{\n  \"env\": { ... }\n}",
                            }),
                    )
                    .into_any_element(),
            )
        };
        let actions = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .justify_end()
            .gap_2()
            .when(window_width >= 600., |actions| {
                actions.child(components::button(
                    "editor-convert",
                    copy_to_app,
                    ButtonTone::Neutral,
                    ButtonSize::Md,
                ))
            })
            .child(self.targeted(
                &["save"],
                components::button(
                    "editor-save",
                    copy.save,
                    ButtonTone::Primary,
                    ButtonSize::Md,
                ),
            ))
            .when(window_width >= 600., |actions| {
                actions.child(components::button(
                    "editor-cancel",
                    cancel,
                    ButtonTone::Neutral,
                    ButtonSize::Md,
                ))
            });
        editor_screen::provider_editor_page(editor_screen::ProviderEditorPage {
            title: SharedString::from(title),
            subtitle: SharedString::from(format!("{subtitle}{}", self.config.app)),
            actions: actions.into_any_element(),
            form_scroll,
            preview,
            form_scrollbar: None,
            modal: None,
            convert_modal: None,
            compact_layout,
            stack_grid: editor_screen::stacks_field_grid(window_width),
        })
    }

    fn providers_page(&self, copy: Copy) -> ochub_ui::gpui::Div {
        let provider = |name: &'static str, models: &'static str, active: bool| {
            gateway_screen::station_card(gateway_screen::StationCard {
                editing: false,
                identity_icon: icons::icon(IconName::Layers, theme::accent(), 16.)
                    .into_any_element(),
                name: SharedString::from(name),
                badges: vec![
                    components::badge(BadgeTone::Teal, "messages").into_any_element(),
                    components::badge(BadgeTone::Mauve, "responses").into_any_element(),
                ],
                base_url: SharedString::from("https://gateway.example.com"),
                endpoint_summary: None,
                website: None,
                model_summary: SharedString::from(models),
                quota: None,
                controls: vec![
                    layout::toggle(active).into_any_element(),
                    components::button(
                        format!("edit-{name}"),
                        copy.edit,
                        ButtonTone::Neutral,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                    components::button(
                        format!("delete-{name}"),
                        copy.delete,
                        ButtonTone::Danger,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                ],
            })
        };
        common_screen::page(
            copy.providers,
            div()
                .flex()
                .gap_2()
                .child(components::button(
                    "station-connection-toggle",
                    "Connection",
                    ButtonTone::Neutral,
                    ButtonSize::Sm,
                ))
                .child(components::button(
                    "station-import-toggle",
                    "Import",
                    ButtonTone::Neutral,
                    ButtonSize::Sm,
                ))
                .child(self.targeted(
                    &["provider-add"],
                    components::button(
                        "station-add",
                        copy.add,
                        ButtonTone::Primary,
                        ButtonSize::Sm,
                    ),
                )),
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p_6()
                .items_center()
                .child(
                    layout::wide_column()
                        .child(self.targeted(
                            &["provider-fields", "detect", "provider-model"],
                            provider("Krill", "39 models configured", true),
                        ))
                        .child(self.targeted(
                            &["model-provider-card"],
                            provider("Team gateway", "3 models · 1 mapping", true),
                        ))
                        .child(provider("Development", "Any model", false)),
                ),
        )
    }

    fn tools_page(&self, page: Page, copy: Copy) -> ochub_ui::gpui::Div {
        let target = match page {
            Page::Mcp => &[
                "mcp-import",
                "mcp-add",
                "mcp-fields",
                "mcp-targets",
                "mcp-save",
                "mcp-sync",
                "mcp-result",
            ][..],
            Page::Skills => &[
                "skill-search",
                "skill-targets",
                "skill-install",
                "skill-result",
            ][..],
            _ => &["session-list", "session-detail"][..],
        };
        let card: ochub_ui::gpui::AnyElement = match page {
            Page::Mcp => tool_screens::mcp_card(tool_screens::McpCard {
                name: SharedString::from("filesystem"),
                endpoint: SharedString::from("npx -y @modelcontextprotocol/server-filesystem"),
                description: Some(SharedString::from("Access selected workspace files")),
                apps_label: SharedString::from("Enabled for: Claude Code, Codex"),
                actions: vec![
                    components::button(
                        "mcp-edit-demo",
                        copy.edit,
                        ButtonTone::Neutral,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                    components::button(
                        "mcp-delete-demo",
                        copy.delete,
                        ButtonTone::Danger,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                ],
                app_toggles: vec![
                    tool_screens::toggle_chip("Claude Code", true).into_any_element(),
                    tool_screens::toggle_chip("Codex", true).into_any_element(),
                    tool_screens::toggle_chip("OpenCode", false).into_any_element(),
                ],
            })
            .into_any_element(),
            Page::Skills => tool_screens::skill_card(tool_screens::SkillCard {
                name: SharedString::from("frontend-design"),
                badge: SharedString::from("Remote"),
                badge_tone: BadgeTone::Teal,
                update_badge: None,
                meta: SharedString::from("openai/skills · Installed today"),
                description: Some(SharedString::from(
                    "Create distinctive, production-grade frontend interfaces.",
                )),
                actions: vec![
                    components::button(
                        "skill-repo-demo",
                        "Repository",
                        ButtonTone::Ghost,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                    components::button(
                        "skill-uninstall-demo",
                        "Uninstall",
                        ButtonTone::Danger,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                ],
                app_toggles: vec![
                    components::badge(BadgeTone::Accent, "Claude Code").into_any_element(),
                    components::badge(BadgeTone::Accent, "Codex").into_any_element(),
                ],
            })
            .into_any_element(),
            _ => tool_screens::session_card(tool_screens::SessionCard {
                provider: SharedString::from("Claude Code"),
                title: SharedString::from("Fix provider routing"),
                snippet: None,
                active_time: Some(SharedString::from("Today, 15:42 · 42 messages")),
                actions: vec![
                    components::button(
                        "session-view-demo",
                        "View",
                        ButtonTone::Primary,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                    components::button(
                        "session-delete-demo",
                        copy.delete,
                        ButtonTone::Danger,
                        ButtonSize::Sm,
                    )
                    .into_any_element(),
                ],
            })
            .into_any_element(),
        };
        let title = match page {
            Page::Mcp => copy.mcp,
            Page::Skills => copy.skills,
            _ => copy.sessions,
        };
        let header_actions = match page {
            Page::Mcp => div()
                .flex()
                .gap_2()
                .child(components::icon_button_tone(
                    "mcp-add",
                    copy.add,
                    IconName::Add,
                    ButtonTone::Primary,
                    ButtonSize::Sm,
                ))
                .child(components::icon_button_tone(
                    "mcp-import",
                    "Import",
                    IconName::Archive,
                    ButtonTone::Neutral,
                    ButtonSize::Sm,
                ))
                .child(components::icon_button_tone(
                    "mcp-sync",
                    "Sync",
                    IconName::Refresh,
                    ButtonTone::Neutral,
                    ButtonSize::Sm,
                )),
            Page::Sessions => div().child(components::icon_button_tone(
                "sessions-refresh",
                "Refresh",
                IconName::Refresh,
                ButtonTone::Neutral,
                ButtonSize::Sm,
            )),
            _ => div(),
        };
        tool_screens::page(
            title,
            header_actions,
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p_6()
                .items_center()
                .child(layout::content_column().child(self.targeted(target, card))),
        )
    }

    fn usage_page(&self, copy: Copy) -> ochub_ui::gpui::Div {
        let stats = usage_screen::summary_grid(vec![
            components::stat_tile(
                Some(IconName::Message),
                theme::green(),
                "Requests",
                "1,248",
                "99.1% successful",
            )
            .into_any_element(),
            components::stat_tile(
                Some(IconName::Diamond),
                theme::peach(),
                "Total cost",
                "$3.42",
                "Last 7 days",
            )
            .into_any_element(),
            components::stat_tile(
                Some(IconName::Layers),
                theme::accent(),
                "Tokens",
                "1.28M",
                "Actual usage",
            )
            .into_any_element(),
            components::stat_tile(
                Some(IconName::Cloud),
                theme::teal(),
                "Cache",
                "86.4%",
                "Input reuse",
            )
            .into_any_element(),
        ]);
        common_screen::page(
            copy.usage,
            components::icon_button_tone(
                "usage-refresh",
                "Refresh",
                IconName::Refresh,
                ButtonTone::Neutral,
                ButtonSize::Sm,
            ),
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p_6()
                .items_center()
                .child(
                    layout::wide_column()
                        .child(self.targeted(
                            &["usage-filter"],
                            components::segmented_readonly(
                                "range",
                                &["24h", "7 days", "30 days"],
                                1,
                            ),
                        ))
                        .child(self.targeted(&["usage-summary"], stats))
                        .child(
                            self.targeted(
                                &["request-detail"],
                                components::card()
                                    .gap_2()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("claude-sonnet-4-6"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::muted())
                                            .child("200 OK · 1,420 input · 382 output · 842 ms"),
                                    ),
                            ),
                        )
                        .child(
                            self.targeted(
                                &["pricing", "pricing-save"],
                                components::card()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Model pricing"),
                                    )
                                    .child(components::button(
                                        "usage-pricing-toggle",
                                        copy.edit,
                                        ButtonTone::Neutral,
                                        ButtonSize::Sm,
                                    )),
                            ),
                        ),
                ),
        )
    }

    fn settings_page(&self, page: Page, copy: Copy) -> ochub_ui::gpui::Div {
        if page == Page::Welcome {
            return layout::page().child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .child(icons::icon(IconName::Desktop, theme::accent(), 42.))
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .child("Welcome to OcHub"),
                    )
                    .child(self.targeted(
                        &["import"],
                        components::button(
                            "import",
                            "Import existing configuration",
                            ButtonTone::Primary,
                            ButtonSize::Md,
                        ),
                    ))
                    .child(components::button(
                        "continue",
                        "Continue",
                        ButtonTone::Neutral,
                        ButtonSize::Md,
                    )),
            );
        }
        let rows = vec![
            layout::row()
                .child(layout::row_label(
                    "Launch at login",
                    Some("Start OcHub automatically".into()),
                ))
                .child(layout::toggle(true))
                .into_any_element(),
            layout::row()
                .child(layout::row_label("Keep running after closing", None))
                .child(layout::toggle(true))
                .into_any_element(),
            layout::row()
                .child(layout::row_label(
                    "Apps and plugins",
                    Some("7 / 9 enabled".into()),
                ))
                .child(components::button(
                    "manage-apps",
                    copy.edit,
                    ButtonTone::Neutral,
                    ButtonSize::Sm,
                ))
                .into_any_element(),
        ];
        common_screen::page(
            copy.settings,
            div()
                .w(px(220.))
                .rounded_md()
                .border_1()
                .border_color(theme::border())
                .bg(theme::inset())
                .px_3()
                .py_2()
                .text_sm()
                .text_color(theme::muted())
                .child("Search settings"),
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p_6()
                .items_center()
                .child(layout::content_column().child(self.targeted(
                    &["apps-settings"],
                    settings_screen::group_block("General", rows),
                ))),
        )
    }

    fn theme_page(&self, copy: Copy) -> ochub_ui::gpui::Div {
        let preview = |light: u32, dark: u32| {
            div()
                .flex()
                .w_full()
                .h(px(92.))
                .child(div().flex_1().h_full().bg(theme::c(light)))
                .child(div().flex_1().h_full().bg(theme::c(dark)))
        };
        let selected = theme_screen::theme_card(theme_screen::ThemeCard {
            preview: preview(0xf4f1ec, 0x242323).into_any_element(),
            selected: true,
            name: SharedString::from("OcHub"),
            description: SharedString::from("Warm neutral built-in theme"),
            badge: components::badge(BadgeTone::Accent, copy.current).into_any_element(),
            actions: div()
                .flex()
                .gap_2()
                .child(components::button(
                    "theme-copy-ochub",
                    copy.copy,
                    ButtonTone::Neutral,
                    ButtonSize::Sm,
                ))
                .child(components::button(
                    "theme-export-ochub",
                    "Export",
                    ButtonTone::Ghost,
                    ButtonSize::Sm,
                ))
                .into_any_element(),
        });
        let ember = theme_screen::theme_card(theme_screen::ThemeCard {
            preview: preview(0xfff2e8, 0x2b211e).into_any_element(),
            selected: false,
            name: SharedString::from("Ember Orange"),
            description: SharedString::from("A vivid user theme"),
            badge: components::badge(BadgeTone::Teal, "User").into_any_element(),
            actions: div()
                .flex()
                .gap_2()
                .child(components::button(
                    "apply-theme",
                    "Apply",
                    ButtonTone::Primary,
                    ButtonSize::Sm,
                ))
                .child(components::button(
                    "theme-edit-ember",
                    copy.edit,
                    ButtonTone::Neutral,
                    ButtonSize::Sm,
                ))
                .into_any_element(),
        });
        let cards = theme_screen::card_row(
            vec![
                div()
                    .flex_1()
                    .min_w_0()
                    .child(self.targeted(&["theme-card"], selected))
                    .into_any_element(),
                div().flex_1().min_w_0().child(ember).into_any_element(),
            ],
            false,
        );
        common_screen::page(
            copy.themes,
            components::icon_button_tone(
                "theme-import",
                "Import",
                IconName::Archive,
                ButtonTone::Neutral,
                ButtonSize::Sm,
            ),
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p_6()
                .items_center()
                .child(
                    layout::wide_column()
                        .child(self.targeted(
                            &["theme-mode"],
                            theme_screen::mode_block(
                                "Appearance mode",
                                "Mode",
                                components::segmented_readonly(
                                    "theme-mode",
                                    &["System", "Light", "Dark"],
                                    0,
                                ),
                                "Theme library",
                            ),
                        ))
                        .child(cards),
                ),
        )
    }

    fn sync_page(&self, copy: Copy) -> ochub_ui::gpui::Div {
        let target = &[
            "sync-fields",
            "sync-test",
            "sync-save",
            "sync-upload",
            "sync-status",
            "sync-stop",
            "sync-restore",
            "sync-result",
            "backup-create",
            "backup-result",
            "backup-restore",
        ];
        let sync_card = components::card()
            .gap_3()
            .child(div().font_weight(FontWeight::BOLD).child("Remote sync"))
            .child(components::segmented_readonly(
                "sync-target",
                &["Off", "WebDAV", "S3"],
                1,
            ))
            .child(
                div()
                    .rounded_md()
                    .bg(theme::inset())
                    .p_3()
                    .text_sm()
                    .child("https://dav.example.com/ochub"),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(components::button(
                        "sync-test",
                        copy.test,
                        ButtonTone::Neutral,
                        ButtonSize::Sm,
                    ))
                    .child(components::button(
                        "sync-save",
                        copy.save,
                        ButtonTone::Primary,
                        ButtonSize::Sm,
                    )),
            );
        let status = components::card()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div().flex().flex_col().child("Last sync").child(
                    div()
                        .text_xs()
                        .text_color(theme::muted())
                        .child("Today, 13:42 · Completed"),
                ),
            )
            .child(components::badge(BadgeTone::Success, copy.enabled));
        common_screen::page(
            copy.settings,
            div(),
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .p_6()
                .items_center()
                .child(
                    layout::content_column()
                        .child(settings_screen::group_block(
                            "Data and backups",
                            vec![self.targeted(target, sync_card).into_any_element()],
                        ))
                        .child(status),
                ),
        )
    }
}

impl Render for SceneView {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.page();
        let copy = Copy::for_locale(&self.config.locale);
        // GPUI's WebGPU canvas uses device pixels, while the desktop editor's
        // responsive thresholds are expressed in logical window pixels. Read
        // the browser viewport directly so Retina backing dimensions cannot
        // incorrectly force the split layout.
        let window_width = web_sys::window()
            .and_then(|browser| browser.inner_width().ok())
            .and_then(|width| width.as_f64())
            .map(|width| width as f32)
            .unwrap_or_else(|| window.viewport_size().width.into());
        let appearance = window.appearance();
        let content = if self.config.scene.starts_with("codex-guide-") {
            self.codex_guide()
        } else {
            match page {
                Page::Connections => self.connection_page(copy),
                Page::Editor => self.editor_page(copy, window_width),
                Page::Providers => self.providers_page(copy),
                Page::Mcp | Page::Skills | Page::Sessions => self.tools_page(page, copy),
                Page::Usage => self.usage_page(copy),
                Page::Settings | Page::Welcome => self.settings_page(page, copy),
                Page::Theme => self.theme_page(copy),
                Page::Sync => self.sync_page(copy),
            }
        };
        // Web cannot resolve desktop system fonts. Use the font bundled by GPUI
        // Web, retaining the registered Noto CJK fallback for Chinese/Japanese.
        let sidebar = div().when(
            window_width >= 1000. && !self.config.scene.starts_with("codex-guide-"),
            |sidebar| sidebar.child(self.sidebar(page, copy, appearance)),
        );
        shell::app_root()
            // Browsers have no native vibrancy backdrop. Composite translucent
            // surfaces over the theme background rather than the dark HTML boot.
            .bg(theme::current().bg.rgba())
            .font_family("IBM Plex Sans")
            .child(shell::app_body(sidebar, content))
    }
}

fn sample_model(app: &str) -> &'static str {
    match app {
        "Codex" => "gpt-5-codex",
        "Grok Build" => "grok-4.5",
        _ => "claude-sonnet-4-6",
    }
}
