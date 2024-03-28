mod context_menu;
mod gist;
mod info_node;
mod level_picker;
mod pill;

use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
    str::FromStr,
};

use gloo::{
    file::File,
    history::{BrowserHistory, History},
    net::http::Request,
};
use web_sys::{js_sys, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::{
    context_menu::{ContextMenu, ContextMenuProvider},
    gist::{CreateGist, CreateGistFile, Gist},
    info_node::InfoNode,
    level_picker::LevelPicker,
    // level_picker::LevelPicker,
};

use log_viewer::{level_filter::LevelFilter, proto::log_level::LogLevel, state::State};

fn gh_token() -> Option<&'static str> {
    std::option_env!("GH_TOKEN")
}

const GH_API_VERSION: &str = "2022-11-28";

fn main() {
    // Enabling trace log slows down the application significantly with large log files
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));

    yew::Renderer::<App>::new().render();
}

#[derive(Properties, Clone, PartialEq)]
struct UploadProps {
    on_upload: Callback<String>,
}

#[function_component]
fn Upload(props: &UploadProps) -> Html {
    let input_ref = use_node_ref();
    let callback = props.on_upload.clone();

    let callback_input_ref = input_ref.clone();

    let onclick = move |_| {
        let input: HtmlTextAreaElement = callback_input_ref.cast().unwrap();
        callback.emit(input.value());
    };

    html! {
        <div class={classes!["w-full", "h-full", "bg-white", "fixed"]}>
        <button onclick={onclick} class="border border-black px-2 py-1 m-3">{"Upload"}</button>
        <div class="w-full h-full p-3">
            <textarea ref={input_ref} class="border border-black w-full h-5/6 p-3"></textarea>
        </div>
    </div>
    }
}

#[function_component]
fn App() -> Html {
    let level_filter = use_state(|| {
        let mut map = HashMap::new();
        let history = BrowserHistory::new();
        let location = history.location();
        let query = location.query::<HashMap<String, String>>().unwrap();
        for (key, value) in query {
            if key.starts_with("filter") {
                let key = key.replace('-', "::").replace("filter::", "");
                map.insert(
                    if key != "filter" { Some(key) } else { None },
                    LogLevel::from_str(&value).unwrap(),
                );
            }
        }
        map.insert(None, LogLevel::Trace);
        LevelFilter::new(map)
    });

    use_effect_with(level_filter.clone(), |level_filter| {
        let history = BrowserHistory::new();
        let location = history.location();

        history
            .push_with_query(location.path(), {
                let mut map = location.query::<HashMap<String, String>>().unwrap();
                let mut remove_keys = vec![];
                for key in map.keys() {
                    if key.starts_with("filter") {
                        remove_keys.push(key.clone());
                    }
                }
                for key in remove_keys {
                    map.remove(&key);
                }
                for (target, filter) in level_filter.matrix() {
                    map.insert(
                        format!(
                            "filter{}",
                            if let Some(target) = target {
                                format!("::{target}")
                            } else {
                                "".into()
                            }
                        )
                        .replace("::", "-"),
                        filter.to_string(),
                    );
                }
                map
            })
            .unwrap();
    });

    let gist = use_state(|| Err(anyhow::anyhow!("Loading file ...")));
    let state = use_state(|| None);
    let show_upload = use_state(|| false);

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
        move |upload_value| {
            let gist_clone = gist_clone.clone();
            let state_clone = state_clone.clone();
            let show_upload = show_upload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let local = || async move {
                    let gist = CreateGist {
                        public: true,
                        files: {
                            let mut map = BTreeMap::new();
                            map.insert(
                                "trace.json".into(),
                                CreateGistFile {
                                    content: upload_value,
                                },
                            );
                            map
                        },
                        description: Some("probe-rs debug trace".into()),
                    };

                    let mut request = Request::post("https://api.github.com/gists")
                        .header("X-GitHub-Api-Version", GH_API_VERSION);

                    if let Some(token) = gh_token() {
                        request = request.header("Authorization", &format!("Bearer {token}"));
                    }

                    let response = request
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
                    let state = gist
                        .current_file()
                        .map(|gist| State::new(&gist.content))
                        .transpose()
                        .unwrap();
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
                gist_clone.set(result.map(|_gist| ()));
                show_upload.set(false);
            });
        }
    };

    let state_clone = state.clone();
    let gist_clone = gist.clone();
    let on_upload_file = {
        move |event: Event| {
            log::debug!("Got event: {event:?}");
            let state_clone = state_clone.clone();
            let gist_clone = gist_clone.clone();

            let input: HtmlInputElement = event.target_unchecked_into();

            if let Some(files) = input.files() {
                let file = js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .map(|v| web_sys::File::from(v.unwrap()))
                    .map(File::from)
                    .next()
                    .unwrap();

                wasm_bindgen_futures::spawn_local(async move {
                    let file = file;
                    let file_content = gloo::file::futures::read_as_text(&file).await;

                    match file_content {
                        Ok(content) => match State::new(&content) {
                            Ok(state) => {
                                state_clone.set(Some(state));
                                gist_clone.set(Ok(()));
                            }
                            Err(e) => {
                                gist_clone.set(Err(anyhow::anyhow!("Error parsing file: {e:?}")))
                            }
                        },
                        Err(err) => {
                            log::error!("Failed to read file: {err}");
                            gist_clone.set(Err(anyhow::anyhow!(err)))
                        }
                    }
                });
            }
        }
    };

    // https://api.github.com/gists/14a826cbe3a884fc3207cde3dfd38817
    let gist_clone = gist.clone();
    let state_clone = state.clone();
    use_effect_with((), move |_| {
        let gist = gist_clone;
        wasm_bindgen_futures::spawn_local(async move {
            let local = move || async {
                let location = BrowserHistory::new()
                    .location()
                    .query::<HashMap<String, String>>()?;

                let hash = location
                    .get("gist")
                    .ok_or_else(|| anyhow::anyhow!("A gist hash must be provided"))?;

                log::debug!("Loading gist {}", hash);
                let mut request = Request::get(&format!("https://api.github.com/gists/{hash}"))
                    .header("X-GitHub-Api-Version", GH_API_VERSION);

                if let Some(token) = gh_token() {
                    request = request.header("Authorization", &format!("Bearer {token}"));
                }

                let response = request
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
                let current_file = gist.current_file().unwrap();

                log::debug!("Using file {} from gist", current_file.filename);

                let content = if current_file.truncated {
                    log::info!(
                        "File {} is truncated, downloading complete file...",
                        current_file.filename
                    );
                    Request::get(&gist.current_file().unwrap().raw_url)
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap()
                } else {
                    current_file.content.clone()
                };

                let state = State::new(&content).ok();
                state_clone.set(state);
            }
            gist.set(result.map(|_gist| ()));
        });
    });

    html! {
        <div class="p-3">
            <ContextMenuProvider>
                <h1 class="text-3xl font-bold">{"Log Viewer"}</h1>
                <ContextMenu />
                if *show_upload {
                    <Upload on_upload={onupload} />
                }
                <div>
                    // <label>{"Search:"}</label>
                    // <input {oninput} value={search_value.to_string()} />
                    // <button onclick={onclick_previous}>{ "<" }</button>
                    // <button onclick={onclick_next}>{ ">" }</button>
                    <LevelPicker level_filter={(*level_filter).clone()} {on_select} />
                    <button onclick={oncreate} class={classes!["ml-3","my-3", "px-2", "py-1", "border", "border-black"]}>{"Create"}</button>
                    <input id="file-upload" type="file" onchange={on_upload_file} class={classes!["ml-3","my-3", "px-2", "py-1", "border", "border-black"]} />
                    <div class="m-3">
                        {match (&*gist, &*state) {
                            (Ok(_gist), Some(state)) => html!{<InfoNode state={Rc::new(state.clone())} node_index={0} level_filter={level_filter.clone()} />},
                            (Err(error), _) => error.to_string().into(),
                            _ => unreachable!()
                        }}
                    </div>
                </div>
            </ContextMenuProvider>
        </div>
    }
}
