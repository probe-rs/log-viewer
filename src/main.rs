mod gist;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use gloo::{
    history::{BrowserHistory, History},
    net::http::Request,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use yew::prelude::*;

use crate::gist::Gist;

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

#[function_component]
fn App() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    let selected_occurrence = use_state(|| 0);
    let total_occurrences = use_state(|| 0);
    let changed_occurrence = use_state(|| false);
    let level_filter = use_state(|| {
        let mut set = HashSet::new();
        set.extend([
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ]);
        set
    });
    let gist = use_state(|| Err(anyhow::anyhow!("Loading file ...")));
    let state = use_state(|| None);
    let search_value = use_state(String::new);
    let onclick_previous = {
        let selected_occurrence = selected_occurrence.clone();
        let changed_occurrence = changed_occurrence.clone();
        let total_occurrences = total_occurrences.clone();
        move |_| {
            let value = *selected_occurrence - 1;
            let value = value.clamp(0, *total_occurrences);
            changed_occurrence.set(true);
            selected_occurrence.set(value);
        }
    };
    let onclick_next = {
        move |_| {
            let value = *selected_occurrence + 1;
            let value = value.clamp(0, *total_occurrences);
            changed_occurrence.set(true);
            selected_occurrence.set(value);
        }
    };
    let oninput = {
        let search_value = search_value.clone();
        move |event: InputEvent| {
            // When events are created the target is undefined, it's only
            // when dispatched does the target get added.
            let target = event.target();
            // Events can bubble so this listener might catch events from child
            // elements which are not of type HtmlInputElement
            let input = target
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();
            search_value.set(input.value());
            // TODO: search()
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

                    log::info!("{:?}", location);
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

    html! {
        <div>
            <label>{"Search:"}</label>
            <input {oninput} value={search_value.to_string()} />
            <button onclick={onclick_previous}>{ "<" }</button>
            <button onclick={onclick_next}>{ ">" }</button>
            <div>{match (&*gist, &*state) {
                (Ok(_gist), Some(state)) => html!{<DrawNode state={state.clone()} node_index={0} level_filter={(*level_filter).clone()} />},
                (Err(error), _) => error.to_string().into(),
                _ => unreachable!()
            }}</div>
        </div>
    }
}

// impl eframe::App for MyApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.horizontal(|ui| {

//                 ui.horizontal(|ui| {
//                     for level in [
//                         LogLevel::Error,
//                         LogLevel::Warn,
//                         LogLevel::Info,
//                         LogLevel::Debug,
//                         LogLevel::Trace,
//                     ] {
//                         if ui
//                             .selectable_label(self.level_filter.contains(&level), level.to_string())
//                             .clicked()
//                         {
//                             if self.level_filter.contains(&level) {
//                                 self.level_filter.remove(&level);
//                             } else {
//                                 self.level_filter.insert(level);
//                             }
//                         }
//                     }
//                 });
//             });
//             egui::ScrollArea::vertical()
//                 .auto_shrink([false, true])
//                 .show(ui, |ui| {
//                     let (_, changed_occurrence) = self.draw_node(ctx, ui, 0, 0);
//                     if changed_occurrence {
//                         self.changed_occurrence = false;
//                     }
//                 });
//         });
//     }
// }

#[derive(Clone, PartialEq, Properties)]
struct DrawNodeProps {
    state: State,
    node_index: usize,
    level_filter: HashSet<LogLevel>,
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
                        if props.level_filter.contains(&event.level) {
                            html! {<div>{message}</div>}
                        } else {
                            html! {<span></span>}
                        }
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

        html! {
            <div class="flex w-full">
                <div class="flex">
                    <svg xmlns="http://www.w3.org/2000/svg" onclick={onclick.clone()} fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class={classes!("w-6", "h-6", if !*collapsed { "block" } else { "hidden" } )}>
                        <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                    </svg>

                    <svg xmlns="http://www.w3.org/2000/svg" {onclick} fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class={classes!("w-6", "h-6", if *collapsed { "block" } else { "hidden" } )}>
                        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
                    </svg>
                </div>
                <div>
                    {level.draw()}
                    {span_title}
                    <span class={classes!(if *collapsed { "block" } else { "hidden" } )}>{body()}</span>
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
    fn draw(&self) -> Html {
        let label = format!("[{self}]");
        let pad = 7 - label.len();
        let color = match self {
            LogLevel::Trace => "white",
            LogLevel::Debug => "blue",
            LogLevel::Info => "green",
            LogLevel::Warn => "yellow",
            LogLevel::Error => "text-red-500",
        };

        let label = format!(
            "{label}{}",
            std::iter::repeat(" ")
                .take(pad)
                .fold(String::new(), |a, b| a + b)
        );
        html! {<span class={color}>{label}</span>}
    }
}
