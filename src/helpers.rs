use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason,
};
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
        .ok_or_else(|| RenderErrorReason::InvalidParamType("[0]: expected non-zero f64"))?;
    let value = h
        .param(1)
        .and_then(|p| p.value().as_f64())
        .ok_or_else(|| RenderErrorReason::InvalidParamType("[1]: expected f64"))?;
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
        .ok_or_else(|| RenderErrorReason::InvalidParamType("[0]: expected u64 (minutes)"))?;
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
        .ok_or_else(|| RenderErrorReason::InvalidParamType("[0]: expected f64 (minutes)"))?;
    out.write(&format!("{}", (percent * 100.0).round()))?;
    Ok(())
}

pub fn format_hours(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = h
        .param(0)
        .and_then(|p| p.value().as_f64())
        .ok_or_else(|| RenderErrorReason::InvalidParamType("[0]: expected f64 (hours)"))?;
    let total_minutes = (value * 60.0) as i64;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if minutes != 0 {
        out.write(&format!("{hours}h{minutes}m"))?;
    } else {
        out.write(&format!("{hours}h"))?;
    }
    Ok(())
}
