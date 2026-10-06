use std::fmt::Write;

use wasm_bindgen::JsCast;

use crate::landing::LandingSection;

const ENTER_START_VIEWPORT_FRACTION: f64 = 0.96;
const ENTER_END_VIEWPORT_FRACTION: f64 = 0.52;
const EXIT_VIEWPORT_FRACTION: f64 = 0.78;
const EXIT_SECTION_FRACTION: f64 = 0.80;
const MOTION_PROPERTY: &str = "--landing-travel";
const PROGRESS_DECIMAL_PLACES: usize = 4;
// Progress is clamped to [-1, 1]; the longest fixed-point value is -1.0000.
const PROGRESS_TEXT_CAPACITY: usize = "-1.".len() + PROGRESS_DECIMAL_PLACES;

pub(crate) struct SectionMotion {
    section: LandingSection,
    style: web_sys::CssStyleDeclaration,
    progress: f64,
    progress_text: String,
}

impl SectionMotion {
    pub(crate) fn bind(section: LandingSection, element: &web_sys::Element) -> Option<Self> {
        let target = match element.query_selector(":scope > .landing-motion") {
            Ok(target) => target,
            Err(source) => {
                log_motion_error("locate motion target", section, &source);
                return None;
            }
        };
        let Some(target) = target.and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            web_sys::console::error_1(
                &format!(
                    "Missing HTML motion target for landing section {}",
                    section.id()
                )
                .into(),
            );
            return None;
        };
        Some(Self {
            section,
            style: target.style(),
            progress: 0.0,
            progress_text: String::with_capacity(PROGRESS_TEXT_CAPACITY),
        })
    }

    pub(crate) fn update(
        &mut self,
        rect: &web_sys::DomRect,
        viewport_top: f64,
        viewport_height: f64,
        should_animate: bool,
    ) {
        let progress = if should_animate {
            compute_motion_progress(rect, viewport_top, viewport_height)
        } else {
            0.0
        };
        if progress == self.progress {
            return;
        }
        self.progress_text.clear();
        if let Err(source) = write!(
            self.progress_text,
            "{progress:.precision$}",
            precision = PROGRESS_DECIMAL_PLACES,
        ) {
            log_motion_error(
                "format motion progress",
                self.section,
                &source.to_string().into(),
            );
            return;
        }
        match self
            .style
            .set_property(MOTION_PROPERTY, &self.progress_text)
        {
            Ok(()) => self.progress = progress,
            Err(source) => log_motion_error("update motion progress", self.section, &source),
        }
    }
}

fn compute_motion_progress(rect: &web_sys::DomRect, top: f64, height: f64) -> f64 {
    if height <= 0.0 || rect.height() <= 0.0 {
        return 0.0;
    }
    let entrance_end = height * ENTER_END_VIEWPORT_FRACTION;
    let entrance_range = height * (ENTER_START_VIEWPORT_FRACTION - ENTER_END_VIEWPORT_FRACTION);
    let entrance = ((rect.top() - top - entrance_end) / entrance_range).clamp(0.0, 1.0);
    if entrance > 0.0 {
        return entrance;
    }
    // Short sections leave only near the top; long sections stay clear while being read.
    let exit_range = (rect.height() * EXIT_SECTION_FRACTION).min(height * EXIT_VIEWPORT_FRACTION);
    -((exit_range - (rect.bottom() - top)) / exit_range).clamp(0.0, 1.0)
}

fn log_motion_error(operation: &str, section: LandingSection, source: &wasm_bindgen::JsValue) {
    web_sys::console::error_2(
        &format!("Failed to {operation} for landing section {}", section.id()).into(),
        source,
    );
}
