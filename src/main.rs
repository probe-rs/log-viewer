mod context_menu;
mod gist;
mod pill;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
};

use context_menu::ContextMenuItemProps;
use gloo::{
    history::{BrowserHistory, History},
    net::http::Request,
};
use pill::Pill;
use serde::{Deserialize, Serialize};
// use wasm_bindgen::JsCast;
// use web_sys::HtmlInputElement;

use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::{
    context_menu::{ContextMenu, ContextMenuProvider},
    gist::{Gist, GistFile},
};

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct State {
    events: Vec<Event>,
    nodes: Vec<Node>,
}

impl State {
    fn new(data: &str) -> Self {
        let events: Result<Vec<Event>, _> = data
            .lines()
            .filter(|l| l.starts_with('{'))
            .map(serde_json::from_str)
            .collect();

        let events = events.expect("Failed to parse events");

        let mut nodes_storage: Vec<Node> = vec![Node {
            index: None,
            children: vec![],
            expanded: true,
        }];

        let mut tree: Vec<usize> = vec![0];

        for (index, event) in events.iter().enumerate() {
            let current_node: usize = *tree.last().expect("at least one node");

            match &event.fields.message[..] {
                "enter" => {
                    let node = Node {
                        index: Some(index),
                        children: vec![],
                        expanded: false,
                    };

                    nodes_storage.push(node);

                    let node_index = nodes_storage.len() - 1;

                    nodes_storage[current_node]
                        .children
                        .push(EventType::Node(node_index));

                    tree.push(node_index);
                }
                "exit" => {
                    let _ = tree.pop();
                }
                "new" => (),
                "close" => (),
                _ => {
                    nodes_storage[current_node]
                        .children
                        .push(EventType::Message(index));
                }
            }
        }

        Self {
            events,
            nodes: nodes_storage,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Node {
    index: Option<usize>,
    /// Indices of all child nodes
    children: Vec<EventType>,
    expanded: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum EventType {
    Message(usize),
    Node(usize),
}

#[derive(Debug, Clone, PartialEq)]
struct LevelFilter {
    matrix: HashMap<String, HashSet<LogLevel>>,
}

impl LevelFilter {
    pub fn show(&self, target: &str, level: &LogLevel) -> bool {
        let selection = if self.matrix.len() == 1 {
            self.matrix.get("")
        } else {
            self.matrix.get(target)
        };

        if let Some(selection) = selection {
            selection.contains(level)
        } else {
            false
        }
    }

    pub fn new(matrix: HashMap<String, HashSet<LogLevel>>) -> Self {
        Self { matrix }
    }

    pub fn add_level(mut self, target: Option<&str>, level: LogLevel) -> Self {
        if let Some(target) = target {
            let target = self.matrix.entry(target.to_string()).or_default();
            target.insert(level);
        } else {
            for target in self.matrix.values_mut() {
                target.insert(level);
            }
        }
        Self {
            matrix: self.matrix,
        }
    }

    pub fn remove_level(mut self, target: Option<&str>, level: &LogLevel) -> Self {
        if let Some(target) = target {
            let target = self.matrix.entry(target.to_string()).or_default();
            target.remove(level);
        } else {
            for target in self.matrix.values_mut() {
                target.remove(level);
            }
        }
        Self {
            matrix: self.matrix,
        }
    }

    pub fn only_level(mut self, target: Option<&str>, level: LogLevel) -> Self {
        let make_target = || {
            let mut filter = HashSet::new();
            filter.insert(level);
            filter
        };

        if let Some(target) = target {
            let target = self.matrix.entry(target.to_string()).or_default();
            *target = make_target();
        } else {
            for target in self.matrix.values_mut() {
                *target = make_target();
            }
        }
        Self {
            matrix: self.matrix,
        }
    }
}

#[function_component]
fn App() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    let level_filter = use_state(|| {
        let mut set = HashSet::new();
        set.extend([
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ]);
        let mut map = HashMap::new();
        map.insert("".to_string(), set);
        LevelFilter::new(map)
    });
    let gist = use_state(|| Err(anyhow::anyhow!("Loading file ...")));
    let state = use_state(|| None);
    let show_upload = use_state(|| false);
    let upload_value = use_state(String::new);
    // let selected_occurrence = use_state(|| 0);
    // let total_occurrences = use_state(|| 0);
    // let changed_occurrence = use_state(|| false);
    // let search_value = use_state(String::new);
    // let onclick_previous = {
    //     let selected_occurrence = selected_occurrence.clone();
    //     let changed_occurrence = changed_occurrence.clone();
    //     let total_occurrences = total_occurrences.clone();
    //     move |_| {
    //         let value = *selected_occurrence - 1;
    //         let value = value.clamp(0, *total_occurrences);
    //         changed_occurrence.set(true);
    //         selected_occurrence.set(value);
    //     }
    // };
    // let onclick_next = {
    //     move |_| {
    //         let value = *selected_occurrence + 1;
    //         let value = value.clamp(0, *total_occurrences);
    //         changed_occurrence.set(true);
    //         selected_occurrence.set(value);
    //     }
    // };
    // let oninput = {
    //     let search_value = search_value.clone();
    //     move |event: InputEvent| {
    //         // When events are created the target is undefined, it's only
    //         // when dispatched does the target get added.
    //         let target = event.target();
    //         // Events can bubble so this listener might catch events from child
    //         // elements which are not of type HtmlInputElement
    //         let input = target
    //             .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
    //             .unwrap();
    //         search_value.set(input.value());
    //         // TODO: search()
    //     }
    // };

    let upload_oninput = {
        let upload_value = upload_value.clone();
        move |event: InputEvent| {
            // When events are created the target is undefined, it's only
            // when dispatched does the target get added.
            let target = event.target();
            // Events can bubble so this listener might catch events from child
            // elements which are not of type HtmlInputElement
            let input = target
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
                .unwrap();
            upload_value.set(input.value());
        }
    };

    let on_select = {
        let level_filter = level_filter.clone();
        move |new_value| level_filter.set(new_value)
    };

    let oncreate = {
        let show_upload = show_upload.clone();
        move |_| show_upload.set(true)
    };

    let onupload = {
        let gist_clone = gist.clone();
        let state_clone = state.clone();
        let show_upload = show_upload.clone();
        move |_| {
            let gist_clone = gist_clone.clone();
            let state_clone = state_clone.clone();
            let show_upload = show_upload.clone();
            let upload_value = upload_value.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let upload_value = upload_value.clone();
                let local = || async move {
                    let token = std::env!("GH_TOKEN");
                    let gist = Gist {
                        id: None,
                        public: true,
                        files: {
                            let mut map = BTreeMap::new();
                            map.insert(
                                "trace.json".into(),
                                GistFile {
                                    name: "trace.json".into(),
                                    content: (*upload_value).clone(),
                                },
                            );
                            map
                        },
                        description: Some("probe-rs debug trace".into()),
                    };
                    let response = Request::post("https://api.github.com/gists")
                        .header("Authorization", &format!("Bearer {token}"))
                        .json(&gist)?
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to load file").context(e))?;
                    if response.status() == 201 {
                        let response: Gist = response
                            .json()
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to load file").context(e))?;
                        Ok(response)
                    } else {
                        anyhow::bail!("Failed to load file with: {}", response.status());
                    }
                };
                let result = local().await;
                if let Ok(gist) = &result {
                    let state = gist.current_file().map(|s| State::new(&s));
                    state_clone.set(state);
                    let history = BrowserHistory::new();

                    history
                        .push_with_query(history.location().path(), {
                            let mut map = HashMap::new();
                            map.insert("gist", gist.id.clone());
                            map
                        })
                        .unwrap();
                }
                gist_clone.set(result);
                show_upload.set(false);
            });
        }
    };

    // https://api.github.com/gists/14a826cbe3a884fc3207cde3dfd38817
    let gist_clone = gist.clone();
    let state_clone = state.clone();
    use_effect_with_deps(
        move |_| {
            let gist = gist_clone;
            wasm_bindgen_futures::spawn_local(async move {
                let local = move || async {
                    let location: HashMap<String, String> =
                        BrowserHistory::new().location().query()?;

                    let hash = location
                        .get("gist")
                        .ok_or_else(|| anyhow::anyhow!("A gist hash must be provided"))?;
                    let token = std::env!("GH_TOKEN");
                    let response = Request::get(&format!("https://api.github.com/gists/{hash}"))
                        .header("Authorization", &format!("Bearer {token}"))
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to load file").context(e))?;
                    if response.status() == 200 {
                        let response: Gist = response
                            .json()
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to load file").context(e))?;
                        Ok(response)
                    } else {
                        anyhow::bail!("Failed to load file with: {}", response.status());
                    }
                };

                let result = local().await;
                if let Ok(gist) = &result {
                    let state = gist.current_file().map(|s| State::new(&s));
                    state_clone.set(state);
                }
                gist.set(result);
            });
        },
        (),
    );

    html! {<ContextMenuProvider>
        <ContextMenu />
        <div class={classes!["w-full", "h-full", "bg-white", if *show_upload { "fixed" } else { "hidden" }]}>
            <button onclick={onupload} class="border border-black px-2 py-1 m-3">{"Upload"}</button>
            <div class="w-full h-full p-3">
                <textarea class="border border-black w-full h-5/6 p-3" oninput={upload_oninput}></textarea>
            </div>
        </div>
        <div>
            // <label>{"Search:"}</label>
            // <input {oninput} value={search_value.to_string()} />
            // <button onclick={onclick_previous}>{ "<" }</button>
            // <button onclick={onclick_next}>{ ">" }</button>
            <LevelPicker level_filter={(*level_filter).clone()} {on_select} />
            <button onclick={oncreate} class={classes!["ml-3","my-3", "px-2", "py-1", "border", "border-black"]}>{"Create"}</button>
            <div class="m-3">
                {match (&*gist, &*state) {
                    (Ok(_gist), Some(state)) => html!{<DrawNode state={state.clone()} node_index={0} level_filter={level_filter.clone()} />},
                    (Err(error), _) => error.to_string().into(),
                    _ => unreachable!()
                }}
            </div>
        </div>
    </ContextMenuProvider>}
}

#[derive(Clone, PartialEq, Properties)]
struct LevelPickerProps {
    level_filter: LevelFilter,
    on_select: Callback<LevelFilter>,
}

#[function_component(LevelPicker)]
fn level_picker(props: &LevelPickerProps) -> Html {
    let onselect = |level| {
        let level_filter = props.level_filter.clone();
        let on_select = props.on_select.clone();
        move |_| {
            let level_filter = level_filter.clone();
            let level_filter = if level_filter.show("", &level) {
                level_filter.remove_level(Some(""), &level)
            } else {
                level_filter.add_level(Some(""), level)
            };
            on_select.emit(level_filter)
        }
    };

    html! {
        [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ].into_iter().map(|level| {
            let checked = props.level_filter.show("", &level);
            let color = level.color();
            html!{<button onclick={onselect(level)} class={classes!["ml-3","my-3", "px-2", "py-1", "border", "border-black",format!("bg-{color}")]}>
                <label class="pr-2">{format!("{level}")}</label>
                <input type="checkbox" {checked} />
            </button>}
        }).collect::<Html>()
    }
}

#[derive(Clone, PartialEq, Properties)]
struct DrawNodeProps {
    state: State,
    node_index: usize,
    level_filter: UseStateHandle<LevelFilter>,
}

#[function_component(DrawNode)]
fn draw_node(props: &DrawNodeProps) -> Html {
    let node = &props.state.nodes[props.node_index];
    let event = node.index.map(|i| &props.state.events[i]);
    let collapsed = use_state(|| node.expanded);

    let onclick = {
        let collapsed = collapsed.clone();
        move |_| {
            collapsed.set(!*collapsed);
        }
    };

    let body = || {
        html! {{
            node.children.iter().map(|child| {
                match child {
                    EventType::Message(message) => {
                        let event = &props.state.events[*message];
                        let message = &event.fields.message;
                        let level = &event.level;
                        let target = &event.target;
                        let hidden = !props.level_filter.show(target, level);
                        html! {<pre class={classes!["py-1", if hidden { "hidden" } else { "block" }]}>
                            {level.draw(props.level_filter.clone())}
                            <span class={classes!["m-1","p-1", "rounded-md", "bg-gray-200"]}>{target}</span>
                            {message}
                        </pre>}

                    }
                    EventType::Node(node_index) => html! {
                        <DrawNode
                            state={props.state.clone()}
                            node_index={node_index}
                            level_filter={props.level_filter.clone()}
                        />
                    },
                }
            }).collect::<Html>()
        }}
    };

    if let Some(event) = event {
        let span_title = event.span.as_ref().unwrap().name.clone();
        let level = event.level;
        let target = &event.target;

        html! {
            <div class="flex w-full py-1">
                <div class="flex flex-col">
                    <svg xmlns="http://www.w3.org/2000/svg" onclick={onclick.clone()} fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class={classes!("w-6", "h-6", if !*collapsed { "block" } else { "hidden" } )}>
                        <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                    </svg>

                    <svg xmlns="http://www.w3.org/2000/svg" {onclick} fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class={classes!("w-6", "h-6", if *collapsed { "block" } else { "hidden" } )}>
                        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
                    </svg>
                    <div class={classes!("grow", "w-3", "border-r", "border-black", if *collapsed { "block" } else { "hidden" } )}></div>
                </div>
                <div>
                    {level.draw(props.level_filter.clone())}
                    <span class={classes!["m-1","p-1", "rounded-md", "bg-gray-200"]}>{target}</span>
                    {span_title}
                    <span class={classes!["pl-3","pt-1", if *collapsed { "block" } else { "hidden" } ]}>{body()}</span>
                </div>
            </div>
        }
    } else {
        body()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Fields {
    message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Span {
    name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Event {
    fields: Fields,
    level: LogLevel,
    span: Option<Span>,
    spans: Option<Vec<Span>>,
    target: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "UPPERCASE")]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = format!("{:?}", self).to_ascii_uppercase();
        write!(f, "{}", text)
    }
}

impl LogLevel {
    fn draw(&self, level_filter: UseStateHandle<LevelFilter>) -> Html {
        let label = format!("[{self}]");
        let pad = 7 - label.len();
        let color = self.color().to_string();
        let level = *self;

        let label = format!(
            "{label}{}",
            std::iter::repeat(" ")
                .take(pad)
                .fold(String::new(), |a, b| a + b)
        );

        let context_menu = vec![
            ContextMenuItemProps {
                callback: {
                    let level_filter = level_filter.clone();
                    Callback::from(move |_| {
                        level_filter.set((*level_filter).clone().only_level(None, level));
                    })
                },
                title: format!("Only show {self}"),
            },
            ContextMenuItemProps {
                callback: Callback::from(move |_| {
                    level_filter.set((*level_filter).clone().remove_level(None, &level));
                }),
                title: format!("Don't show {self}"),
            },
        ];

        html! {<Pill
                {color}
                {context_menu}
            >
                {label}
        </Pill>}
    }

    fn color(&self) -> &str {
        match self {
            LogLevel::Trace => "gray-500",
            LogLevel::Debug => "blue-500",
            LogLevel::Info => "green-500",
            LogLevel::Warn => "orange-500",
            LogLevel::Error => "red-500",
        }
    }
}
