//! Diagnostic-only startup comparison; never used by production launch APIs.
use terminalhud_windows_terminal::LaunchPlan;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Presentation {
    Fullscreen,
    WindowedControl,
}
impl Presentation {
    pub(crate) fn parse(show: u16, flags: &[String]) -> Result<Self, &'static str> {
        match flags {
            [] => Ok(Self::Fullscreen),
            [flag] if flag == "--windowed-control" && show == 8 => Ok(Self::WindowedControl),
            _ => Err("only show8 accepts the sole trailing --windowed-control flag"),
        }
    }
    pub(crate) fn arguments(self, plan: &LaunchPlan) -> Result<Vec<String>, &'static str> {
        let mut arguments = plan.arguments();
        if self == Self::WindowedControl {
            let indices: Vec<_> = arguments
                .iter()
                .enumerate()
                .filter_map(|(index, value)| (value == "--fullscreen").then_some(index))
                .collect();
            let [index] = indices.as_slice() else {
                return Err("diagnostic baseline must contain exactly one fullscreen flag");
            };
            arguments.remove(*index);
        }
        Ok(arguments)
    }
}
