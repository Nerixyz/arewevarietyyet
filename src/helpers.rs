use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use std::time::Duration;

pub fn bar_width(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let total = h
        .param(0)
        .and_then(|p| p.value().as_f64().filter(|v| *v != 0.0))
        .ok_or_else(|| RenderError::new("Expected non-zero total"))?;
    let value = h
        .param(1)
        .and_then(|p| p.value().as_f64())
        .ok_or_else(|| RenderError::new("Expected value"))?;
    out.write(&(value * 100.0 / total).to_string())?;
    Ok(())
}

pub fn humanize_min(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let minutes = h
        .param(0)
        .and_then(|p| p.value().as_u64())
        .ok_or_else(|| RenderError::new("Expected minutes"))?;
    out.write(&humantime::format_duration(Duration::from_secs(minutes * 60)).to_string())?;
    Ok(())
}

pub fn rounded_percent(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let percent = h
        .param(0)
        .and_then(|p| p.value().as_f64())
        .ok_or_else(|| RenderError::new("Expected minutes"))?;
    out.write(&format!("{}", (percent * 100.0).round()))?;
    Ok(())
}
