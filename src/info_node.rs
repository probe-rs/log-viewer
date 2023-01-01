use yew::{
    classes, function_component, html, use_state, Callback, Html, Properties, UseStateHandle,
};

use crate::{
    context_menu::ContextMenuItemProps,
    level_filter::LevelFilter,
    pill::Pill,
    proto::log_level::LogLevel,
    state::{EventType, State},
};

#[derive(Clone, PartialEq, Properties)]
pub struct InfoNodeProps {
    pub state: State,
    pub node_index: usize,
    pub level_filter: UseStateHandle<LevelFilter>,
}

#[function_component(InfoNode)]
pub fn info_node(props: &InfoNodeProps) -> Html {
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
                        let hidden = !props.level_filter.show(Some(target.clone()), level);

                        let context_menu = vec![
                            ContextMenuItemProps {
                                callback: {
                                    let level_filter = props.level_filter.clone();
                                    let target = target.clone();
                                    let level = *level;
                                    Callback::from(move |_| {
                                        level_filter
                                            .set((*level_filter)
                                            .clone()
                                            .set_level(None, LogLevel::None)
                                            .set_level(Some(target.clone()), level));
                                    })
                                },
                                title: format!("Only show {target}"),
                            },
                            ContextMenuItemProps {
                                callback: {
                                    let level_filter = props.level_filter.clone();
                                    let target = target.clone();
                                    Callback::from(move |_| {
                                    level_filter.set((*level_filter).clone().set_level(Some(target.clone()), LogLevel::None))
                                })},
                                title: format!("Don't show {target}"),
                            },
                        ];

                        html! {<pre class={classes!["pl-6", "py-1", if hidden { "hidden" } else { "block" }]}>
                            {level.draw(props.level_filter.clone())}
                            <Pill color="gray-200" {context_menu}><span class={classes!["m-1","p-1", "rounded-md", "bg-gray-200"]}>{target}</span></Pill>
                            {message}
                        </pre>}

                    }
                    EventType::Node(node_index) => html! {
                        <InfoNode
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
                    <span class={classes!["pt-1", if *collapsed { "block" } else { "hidden" } ]}>{body()}</span>
                </div>
            </div>
        }
    } else {
        body()
    }
}
