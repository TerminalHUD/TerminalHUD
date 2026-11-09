use super::{Assessment, MAX_GAP_US, Sample, Span, Stage, Window};
/// Setup can change foreground. Startup permits only the sink or verified HUD;
/// an owned return during startup is recorded without attributing its cause.
/// Active inspection requires HUD foreground; recovery inspection requires sink.
/// Quit permits the existing owned HUD -> sink restoration, never a third window.
pub(crate) fn classify(
    samples: &[Sample],
    sink: Window,
    target: Option<Window>,
    span: Span,
    truncated: bool,
) -> Assessment {
    let protected: Vec<_> = samples
        .iter()
        .filter(|s| {
            s.stage != Stage::Setup
                && s.elapsed_us >= span.start_us
                && s.elapsed_us <= span.end_us
                && (span.through_quit || s.stage != Stage::Quit)
        })
        .collect();
    let mut result = Assessment {
        observed_input_activity: false,
        unexpected_foreground: false,
        insufficient_observation: truncated || protected.is_empty() || target.is_none(),
    };
    let baseline = protected.first().and_then(|s| s.input_tick);
    let mut previous = span.start_us;
    let mut phase = 0;
    let (mut startup, mut inspected, mut quit) = (false, false, false);
    for sample in protected {
        let next_phase = match sample.stage {
            Stage::Setup => 0,
            Stage::Startup => 1,
            Stage::InspectActive | Stage::InspectRecovered => 2,
            Stage::Quit => 3,
        };
        result.insufficient_observation |= next_phase < phase;
        phase = next_phase;
        if sample.elapsed_us < previous
            || sample.elapsed_us.saturating_sub(previous) > MAX_GAP_US
            || sample.elapsed_us < span.start_us
            || sample.elapsed_us > span.end_us
            || sample.input_tick.is_none()
            || sample.foreground.is_none()
        {
            result.insufficient_observation = true;
        }
        if let (Some(first), Some(current)) = (baseline, sample.input_tick) {
            result.observed_input_activity |= current != first;
        }
        let is_sink = sample.foreground == Some(sink);
        let is_target = target.is_some() && sample.foreground == target;
        let permitted = match sample.stage {
            Stage::Setup => unreachable!(),
            Stage::Startup => {
                startup = true;
                is_sink || is_target
            }
            Stage::InspectActive => {
                inspected = true;
                is_target
            }
            Stage::InspectRecovered => {
                inspected = true;
                is_sink
            }
            Stage::Quit => {
                quit = true;
                is_sink || is_target
            }
        };
        result.unexpected_foreground |= sample.foreground.is_some() && !permitted;
        previous = sample.elapsed_us;
    }
    result.insufficient_observation |= !startup
        || !inspected
        || (span.through_quit && !quit)
        || span.end_us < previous
        || span.end_us.saturating_sub(previous) > MAX_GAP_US;
    result
}
