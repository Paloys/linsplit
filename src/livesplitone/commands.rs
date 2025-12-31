use serde::Serializer;
use std::time::Duration;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct TimeSpan(Duration);

#[derive(Clone, serde_derive::Serialize)]
#[serde(tag = "command", rename_all = "camelCase")]
pub enum Command {
    Start,
    Split,
    SplitOrStart,
    #[serde(rename_all = "camelCase")]
    Reset {
        #[serde(skip_serializing_if = "Option::is_none")]
        save_attempt: Option<bool>,
    },
    UndoSplit,
    SkipSplit,
    TogglePauseOrStart,
    Pause,
    Resume,
    UndoAllPauses,
    SwitchToPreviousComparison,
    SwitchToNextComparison,
    InitializeGameTime,
    SetGameTime {
        /// The time to set the game time to.
        #[serde(serialize_with = "serialize_time_span")]
        time: TimeSpan,
    },
    PauseGameTime,
    ResumeGameTime,
    GetCurrentState,
    Ping,
}

impl TimeSpan {
    pub const fn to_seconds_and_subsec_nanoseconds(&self) -> (i64, i32) {
        (self.0.as_secs() as i64, self.0.subsec_nanos() as i32)
    }

    pub fn from_seconds(seconds: f64) -> Self {
        Self(Duration::from_secs_f64(seconds))
    }
}

fn serialize_time_span<S: Serializer>(
    time_span: &TimeSpan,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let (secs, nanos) = time_span.to_seconds_and_subsec_nanoseconds();
    serializer.collect_str(&format_args!("{secs}.{:09}", nanos.abs()))
}

#[derive(serde_derive::Deserialize, Debug)]
#[serde(tag = "state", content = "index")]
enum State {
    NotRunning,
    Running(usize),
    Paused(usize),
    Ended,
}

#[derive(serde_derive::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum CommandResult<T, E> {
    Success(T),
    Error(E),
}

#[derive(serde_derive::Deserialize, Debug)]
#[serde(untagged)]
pub enum Response {
    None,
    String(String),
    State(State),
}

#[derive(serde_derive::Deserialize, Debug)]
#[serde(tag = "code")]
pub enum CommandError {
    InvalidCommand {
        message: String,
    },
    InvalidIndex,
    #[serde(untagged)]
    Timer {
        code: EventError,
    },
}

#[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
#[non_exhaustive]
#[serde(tag = "event")]
pub enum Event {
    Started,
    Splitted,
    Finished,
    Reset,
    SplitUndone,
    SplitSkipped,
    Paused,
    Resumed,
    PausesUndone,
    PausesUndoneAndResumed,
    ComparisonChanged,
    TimingMethodChanged,
    GameTimeInitialized,
    GameTimeSet,
    GameTimePaused,
    GameTimeResumed,
    LoadingTimesSet,
    CustomVariableSet,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
#[non_exhaustive]
pub enum EventError {
    Unsupported,
    Busy,
    RunAlreadyInProgress,
    NoRunInProgress,
    RunFinished,
    NegativeTime,
    CantSkipLastSplit,
    CantUndoFirstSplit,
    AlreadyPaused,
    NotPaused,
    ComparisonDoesntExist,
    GameTimeAlreadyInitialized,
    GameTimeAlreadyPaused,
    GameTimeNotPaused,
    CouldNotParseTime,
    TimerPaused,
    RunnerDecidedAgainstReset,
    #[serde(other)]
    Unknown,
}
