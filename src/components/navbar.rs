use once_cell::sync::Lazy;
use yew::prelude::*;
use yew_autoprops::autoprops;
use yew_icons::{Icon, IconId};
use yew_router::hooks::use_navigator;

use super::{home::RepoUrlBar, routes::GoHome, REPOSITORY};

pub const NAVBAR_BUTTON_CLASSES: Lazy<Classes> = Lazy::new(|| {
    classes!(
        "appearance-none",
        "bg-teal-500",
        "dark:bg-teal-900",
        "border",
        "border-teal-500",
        "text-teal-50",
        "text-sm",
        "rounded",
        "p-1",
        "focus:ring-blue-500",
        "focus:border-blue-500",
        "block",
    )
});

#[autoprops]
#[function_component(Logo)]
pub fn logo() -> HtmlResult {
    let navigator = use_navigator();
    Ok(html! {
        <GoHome navigator={navigator}>
            <span class={classes!("font-semibold", "text-xl", "tracking-tight")}>
                { "Toukei" }
            </span>
        </GoHome>
    })
}

#[autoprops]
#[function_component(Navbar)]
pub fn navbar() -> HtmlResult {
    Ok(html! {
        <nav class={classes!("flex", "items-center", "flex-wrap", "text-teal-50", "bg-teal-600", "dark:bg-teal-950", "py-3", "px-6")}>
            <div class={classes!("flex", "justify-between", "items-center", "w-full")}>
                <div class={classes!("inline-block", "text-center")}>
                    <Logo/>
                </div>
                <div class={classes!("inline-block", "invisible", "md:visible", "shrink", "w-full", "max-w-screen-md")}>
                    <RepoUrlBar/>
                </div>
                <div class={classes!("inline-block", "text-right", "text-teal-200", "text-sm")}>
                    <a href={REPOSITORY}>
                        <Icon icon_id={IconId::HeroiconsOutlineInformationCircle} title={"Information"}/>
                    </a>
                </div>
            </div>
        </nav>
    })
}
