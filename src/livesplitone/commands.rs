use std::borrow::Cow;

#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize)]
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
    PauseGameTime,
    ResumeGameTime,
    GetCurrentState,
    Ping,
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

#[derive(
    Debug, serde_derive::Serialize, serde_derive::Deserialize,
)]
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
