//! Portable HudSwitch state machine and synchronous backend contract.
//!
//! A controller exclusively owns its target and backend. Hosts serialize event
//! delivery through a queue or a mutex; [`Controller::dispatch`] requires an
//! exclusive borrow for the complete transition. Backend callbacks enqueue new
//! events rather than recursively dispatching them. See `docs/hudswitch-core.md`.

#![forbid(unsafe_code)]

/// A visible state that can be remembered while the target is hidden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibleState {
    Active,
    Passive,
}

/// Logical state, separate from target availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Active,
    Passive,
    Hidden { return_state: VisibleState },
}

/// Requested backend presentation; hidden-state memory stays in the core.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presentation {
    Active,
    Passive,
    Hidden,
}

impl State {
    fn presentation(self) -> Presentation {
        match self {
            Self::Active => Presentation::Active,
            Self::Passive => Presentation::Passive,
            Self::Hidden { .. } => Presentation::Hidden,
        }
    }
}

/// Complete operations the backend can support on this platform/session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    pub launch: bool,
    pub active: bool,
    pub passive: bool,
    pub hidden: bool,
}

impl Capabilities {
    pub const ALL: Self = Self {
        launch: true,
        active: true,
        passive: true,
        hidden: true,
    };

    fn supports(self, presentation: Presentation) -> bool {
        match presentation {
            Presentation::Active => self.active,
            Presentation::Passive => self.passive,
            Presentation::Hidden => self.hidden,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Absent,
    Managed,
    Unmanaged,
}

/// Controller-local generation for rejecting stale lifecycle notifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetId(u64);

/// Confirmed logical state. `Managed` with no state means recovery is required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub availability: Availability,
    pub state: Option<State>,
    pub target_id: Option<TargetId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    InteractionToggle,
    VisibilityToggle,
    RequestPassive,
    FocusLost(TargetId),
    TargetClosed(TargetId),
    Quit,
}

/// Observable backend facts used to confirm the requested invariant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub visible: bool,
    pub topmost: bool,
    pub click_through: bool,
    pub focused: bool,
}

impl Observation {
    fn confirms(self, presentation: Presentation) -> bool {
        match presentation {
            Presentation::Active => {
                self.visible && self.topmost && !self.click_through && self.focused
            }
            Presentation::Passive => {
                self.visible && self.topmost && self.click_through && !self.focused
            }
            Presentation::Hidden => !self.visible && !self.focused,
        }
    }
}

/// Acquired target and its original state, captured before controller mutation.
/// Dropping either token must not terminate or close the target application.
pub struct Acquired<Target, Original> {
    pub target: Target,
    pub original: Original,
}

/// Platform contract. Methods must validate identity before touching a target.
/// No method may change terminal appearance, opacity, or unrelated windows.
pub trait Backend {
    type Target;
    type Focus;
    type Original;
    type Error;

    fn capabilities(&self) -> Capabilities;

    /// Launch/acquire only the intended target and capture original owned state.
    /// Passive intent must not steal focus, even briefly. On error, the backend
    /// retains responsibility for leaving any created target ordinary and usable.
    fn acquire(
        &mut self,
        intent: VisibleState,
    ) -> Result<Acquired<Self::Target, Self::Original>, Self::Error>;

    fn is_valid(&mut self, target: &Self::Target) -> Result<bool, Self::Error>;
    fn foreground(&mut self) -> Result<Option<Self::Focus>, Self::Error>;

    /// Apply visibility, topmost, and input style changes, preserving all bits
    /// outside the backend's ownership. This does not activate another window.
    fn present(
        &mut self,
        target: &Self::Target,
        presentation: Presentation,
    ) -> Result<(), Self::Error>;

    fn activate(&mut self, target: &Self::Target) -> Result<(), Self::Error>;

    /// Attempt the saved window once, only if it is still valid and eligible.
    fn restore_focus(&mut self, focus: &Self::Focus) -> Result<(), Self::Error>;

    /// Relinquish target focus only if it still owns focus; preserve a foreground
    /// window the user selected meanwhile. Let the OS choose when no saved
    /// foreground can be restored; do not force or retry another application.
    fn release_focus_if_owned(&mut self, target: &Self::Target) -> Result<(), Self::Error>;

    fn observe(&mut self, target: &Self::Target) -> Result<Observation, Self::Error>;

    /// Restore only changes owned by this backend from the original snapshot.
    /// Leave the target open and usable. Returning success certifies completion.
    fn restore_original(
        &mut self,
        target: &Self::Target,
        original: &Self::Original,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Capabilities,
    Acquire,
    Validate,
    CaptureFocus,
    Present,
    Activate,
    RestoreFocus,
    ReleaseFocus,
    Observe,
    RestoreOriginal,
}

#[derive(Debug)]
pub enum Cause<E> {
    Backend(E),
    Unsupported,
    InvariantMismatch(Observation),
    RecoveryRequired,
    Stopped,
    GenerationExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recovery {
    NotNeeded,
    Passive,
    Hidden,
    Absent,
    Unconfirmed,
}

#[derive(Debug)]
pub struct AttemptFailure<E> {
    pub stage: Stage,
    pub cause: Cause<E>,
}

#[derive(Debug)]
pub struct TransitionError<E> {
    pub failure: AttemptFailure<E>,
    pub recovery: Recovery,
    pub recovery_failures: Vec<AttemptFailure<E>>,
}

/// Failure to restore a previous foreground is a warning after safe completion.
#[derive(Debug)]
pub struct Outcome<E> {
    pub status: Status,
    pub focus_restore_error: Option<E>,
}

struct Managed<B: Backend> {
    target: B::Target,
    original: B::Original,
    focus: Option<B::Focus>,
    state: Option<State>,
    id: TargetId,
}

pub struct Controller<B: Backend> {
    backend: B,
    managed: Option<Managed<B>>,
    stopped: bool,
    generation: u64,
}

impl<B: Backend> Controller<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            managed: None,
            stopped: false,
            generation: 0,
        }
    }

    pub fn status(&self) -> Status {
        Status {
            availability: if self.stopped {
                Availability::Unmanaged
            } else if self.managed.is_some() {
                Availability::Managed
            } else {
                Availability::Absent
            },
            state: self.managed.as_ref().and_then(|target| target.state),
            target_id: self.managed.as_ref().map(|target| target.id),
        }
    }

    /// Complete one transition before accepting the next event.
    pub fn dispatch(
        &mut self,
        event: Event,
    ) -> Result<Outcome<B::Error>, TransitionError<B::Error>> {
        let mut target = self.managed.take();
        let result = self.dispatch_inner(&mut target, event);
        self.managed = target;
        result.map(|warning| Outcome {
            status: self.status(),
            focus_restore_error: warning,
        })
    }

    /// Explicitly retry safe Passive, then Hidden, for an unconfirmed target.
    /// Retains ownership and the original snapshot if recovery remains impossible.
    pub fn recover(&mut self) -> Result<Outcome<B::Error>, TransitionError<B::Error>> {
        let mut target = self.managed.take();
        let mut failures = Vec::new();
        let recovery = self.make_safe(&mut target, true, &mut failures);
        self.managed = target;
        match recovery {
            Recovery::Unconfirmed => Err(TransitionError {
                failure: AttemptFailure {
                    stage: Stage::Observe,
                    cause: Cause::RecoveryRequired,
                },
                recovery,
                recovery_failures: failures,
            }),
            _ => Ok(Outcome {
                status: self.status(),
                focus_restore_error: None,
            }),
        }
    }

    fn dispatch_inner(
        &mut self,
        target: &mut Option<Managed<B>>,
        event: Event,
    ) -> Result<Option<B::Error>, TransitionError<B::Error>> {
        if self.stopped {
            return if event == Event::Quit {
                Ok(None)
            } else {
                Err(Self::error(Stage::Validate, Cause::Stopped))
            };
        }
        if let Event::TargetClosed(id) = event {
            if target.as_ref().is_some_and(|current| current.id == id) {
                *target = None;
            }
            return Ok(None);
        }
        if let Event::FocusLost(id) = event
            && !target.as_ref().is_some_and(|current| current.id == id)
        {
            return Ok(None);
        }
        if let Some(current) = target.as_mut() {
            match self.backend.is_valid(&current.target) {
                Ok(false) => *target = None,
                Ok(true) => {}
                Err(error) => {
                    current.state = None;
                    let mut failure = Self::error(Stage::Validate, Cause::Backend(error));
                    failure.recovery = Recovery::Unconfirmed;
                    return Err(failure);
                }
            }
        }
        if event == Event::Quit {
            if let Some(current) = target.as_mut() {
                current.state = None;
                if let Err(error) = self
                    .backend
                    .restore_original(&current.target, &current.original)
                {
                    return Err(self.failed(
                        target,
                        Stage::RestoreOriginal,
                        Cause::Backend(error),
                        false,
                    ));
                }
            }
            *target = None;
            self.stopped = true;
            return Ok(None);
        }
        let state = target.as_ref().and_then(|current| current.state);
        if target.is_some() && state.is_none() {
            let mut failure = Self::error(Stage::Observe, Cause::RecoveryRequired);
            failure.recovery = Recovery::Unconfirmed;
            return Err(failure);
        }
        let requested = match (event, state) {
            (Event::InteractionToggle, Some(State::Active)) => State::Passive,
            (Event::InteractionToggle, _) => State::Active,
            (Event::VisibilityToggle, Some(State::Active)) => State::Hidden {
                return_state: VisibleState::Active,
            },
            (Event::VisibilityToggle, Some(State::Passive)) => State::Hidden {
                return_state: VisibleState::Passive,
            },
            (
                Event::VisibilityToggle,
                Some(State::Hidden {
                    return_state: VisibleState::Active,
                }),
            ) => State::Active,
            (
                Event::VisibilityToggle,
                Some(State::Hidden {
                    return_state: VisibleState::Passive,
                }),
            ) => State::Passive,
            (Event::VisibilityToggle, None) => State::Passive,
            (Event::RequestPassive | Event::FocusLost(_), Some(State::Active)) => State::Passive,
            _ => return Ok(None),
        };
        let capabilities = self.backend.capabilities();
        if !capabilities.supports(requested.presentation())
            || (target.is_none() && !capabilities.launch)
            || (requested == State::Active && !capabilities.passive && !capabilities.hidden)
        {
            return Err(Self::error(Stage::Capabilities, Cause::Unsupported));
        }
        // Capture the external foreground before acquisition can activate a new window.
        let focus = if requested == State::Active {
            self.backend
                .foreground()
                .map_err(|error| Self::error(Stage::CaptureFocus, Cause::Backend(error)))?
        } else {
            None
        };
        if target.is_none() {
            let generation = self
                .generation
                .checked_add(1)
                .ok_or_else(|| Self::error(Stage::Acquire, Cause::GenerationExhausted))?;
            let intent = if requested == State::Active {
                VisibleState::Active
            } else {
                VisibleState::Passive
            };
            let acquired = self
                .backend
                .acquire(intent)
                .map_err(|error| Self::error(Stage::Acquire, Cause::Backend(error)))?;
            self.generation = generation;
            *target = Some(Managed {
                target: acquired.target,
                original: acquired.original,
                focus: None,
                state: None,
                id: TargetId(generation),
            });
        }
        let current = target.as_mut().expect("target was acquired");
        if requested == State::Active {
            current.focus = focus;
        }
        let preserve_foreground = matches!(event, Event::FocusLost(_))
            || (requested != State::Active && state != Some(State::Active));
        self.apply(current, requested, preserve_foreground)
            .map_err(|failure| {
                self.failed(target, failure.stage, failure.cause, preserve_foreground)
            })
    }

    fn apply(
        &mut self,
        target: &mut Managed<B>,
        state: State,
        preserve_foreground: bool,
    ) -> Result<Option<B::Error>, AttemptFailure<B::Error>> {
        target.state = None;
        let presentation = state.presentation();
        self.backend
            .present(&target.target, presentation)
            .map_err(|error| AttemptFailure {
                stage: Stage::Present,
                cause: Cause::Backend(error),
            })?;
        let mut warning = None;
        if state == State::Active {
            self.backend
                .activate(&target.target)
                .map_err(|error| AttemptFailure {
                    stage: Stage::Activate,
                    cause: Cause::Backend(error),
                })?;
        } else {
            let focus = target.focus.take();
            if !preserve_foreground && let Some(focus) = focus {
                warning = self.backend.restore_focus(&focus).err();
            }
            self.backend
                .release_focus_if_owned(&target.target)
                .map_err(|error| AttemptFailure {
                    stage: Stage::ReleaseFocus,
                    cause: Cause::Backend(error),
                })?;
        }
        let observation = self
            .backend
            .observe(&target.target)
            .map_err(|error| AttemptFailure {
                stage: Stage::Observe,
                cause: Cause::Backend(error),
            })?;
        if !observation.confirms(presentation) {
            return Err(AttemptFailure {
                stage: Stage::Observe,
                cause: Cause::InvariantMismatch(observation),
            });
        }
        target.state = Some(state);
        Ok(warning)
    }

    fn make_safe(
        &mut self,
        target: &mut Option<Managed<B>>,
        preserve_foreground: bool,
        failures: &mut Vec<AttemptFailure<B::Error>>,
    ) -> Recovery {
        let Some(current) = target.as_mut() else {
            return Recovery::Absent;
        };
        current.state = None;
        match self.backend.is_valid(&current.target) {
            Ok(false) => {
                *target = None;
                return Recovery::Absent;
            }
            Err(error) => {
                failures.push(AttemptFailure {
                    stage: Stage::Validate,
                    cause: Cause::Backend(error),
                });
                return Recovery::Unconfirmed;
            }
            Ok(true) => {}
        }
        let capabilities = self.backend.capabilities();
        for (state, recovery) in [
            (State::Passive, Recovery::Passive),
            (
                State::Hidden {
                    return_state: VisibleState::Passive,
                },
                Recovery::Hidden,
            ),
        ] {
            if capabilities.supports(state.presentation()) {
                match self.apply(current, state, preserve_foreground) {
                    Ok(warning) => {
                        if let Some(error) = warning {
                            failures.push(AttemptFailure {
                                stage: Stage::RestoreFocus,
                                cause: Cause::Backend(error),
                            });
                        }
                        return recovery;
                    }
                    Err(error) => failures.push(error),
                }
            }
        }
        Recovery::Unconfirmed
    }

    fn failed(
        &mut self,
        target: &mut Option<Managed<B>>,
        stage: Stage,
        cause: Cause<B::Error>,
        preserve_foreground: bool,
    ) -> TransitionError<B::Error> {
        let mut recovery_failures = Vec::new();
        let recovery = self.make_safe(target, preserve_foreground, &mut recovery_failures);
        TransitionError {
            failure: AttemptFailure { stage, cause },
            recovery,
            recovery_failures,
        }
    }

    fn error(stage: Stage, cause: Cause<B::Error>) -> TransitionError<B::Error> {
        TransitionError {
            failure: AttemptFailure { stage, cause },
            recovery: Recovery::NotNeeded,
            recovery_failures: Vec::new(),
        }
    }
}
