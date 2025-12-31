use bitflags::bitflags;
bitflags! {
    pub struct AutoSplitterChapterFlags : u32 {
        const CHAPTER_STARTED = 1 << 0;
        const CHAPTER_COMPLETE = 1 << 1;
        const CHAPTER_CASSETTE = 1 << 2;
        const CHAPTER_HEART = 1 << 3;
        const GRABBED_GOLDEN = 1 << 4;

        const TIMER_ACTIVE = 1 << 31;
    }
}

bitflags! {
    pub struct AutoSplitterFileFlags : u32 {
        const IS_DEBUG = 1 << 0;
        const ASSIST_MODE = 1 << 1;
        const VARIANTS_MODE = 1 << 2;

        const STARTING_NEW_FILE = 1 << 30;
        const FILE_ACTIVE = 1 << 31;
    }
}
