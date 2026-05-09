use dialoguer::Input;
use crate::config::I18n;
use crate::core::estimation;
use crate::display;
use anyhow::Result;

pub fn run(i18n: &I18n, usage_type: &str) -> Result<()> {
    match usage_type {
        "book" => {
            let pages: u32 = Input::new().with_prompt(i18n.t("estimator.pages")).default(100).interact()?;
            let tokens = estimation::tokens_book(pages);
            let (min, max) = estimation::time_minutes(tokens, estimation::performance(14, true));
            println!("  {}", i18n.t_with_vars("estimation.tokens", &[("tokens", &display::format_number(tokens))]));
            println!("  {}", i18n.t_with_vars("estimation.time_range", &[
                ("min", &estimation::format_duration(min)), ("max", &estimation::format_duration(max))
            ]));
        }
        "code" => {
            let loc: u32 = Input::new().with_prompt(i18n.t("estimator.loc")).default(10000).interact()?;
            let tokens = estimation::tokens_code(loc);
            let (min, max) = estimation::time_minutes(tokens, estimation::performance(14, true));
            println!("  {}", i18n.t_with_vars("estimation.tokens", &[("tokens", &display::format_number(tokens))]));
            println!("  {}", i18n.t_with_vars("estimation.time_range", &[
                ("min", &estimation::format_duration(min)), ("max", &estimation::format_duration(max))
            ]));
        }
        _ => {}
    }
    Ok(())
}