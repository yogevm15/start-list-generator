use std::cmp::max;
use std::collections::VecDeque;
use std::ops::{Add, Div};

use chrono::Duration;
use rand::prelude::SliceRandom;
use rand::thread_rng;

type Minutes = usize;
struct Competitor {
    name: String,
}

struct CompetitorWithOffset {
    competitor: Competitor,
    offset: Minutes,
}

struct Window {
    duration: Minutes,
    competitors: VecDeque<Competitor>,
}

fn generate_startlist(
    mut windows: Vec<Window>,
    spacing_threshold: Minutes,
    min_spacing: Minutes,
) -> Vec<CompetitorWithOffset> {
    let mut competitors_count = 0;
    for window in windows.iter_mut() {
        window
            .competitors
            .make_contiguous()
            .shuffle(&mut thread_rng());
        competitors_count += window.competitors.len();
    }

    for i in 0..windows.len() {
        stabilize_window(&mut windows, i, spacing_threshold);
    }

    let mut competitors = Vec::with_capacity(competitors_count);
    let mut curr_start = 0;
    let mut windows_curr_start = 0;
    let windows_len = windows.len();
    for (i, window) in windows.into_iter().enumerate() {
        if window.competitors.len() as i32 != 0 {
            let space = if i == windows_len - 1 {
                // Last window (inclusive duration)
                calculate_window_space(window.duration, window.competitors.len() - 1)
            } else {
                // Normal window (exclusive duration)
                calculate_window_space(window.duration, window.competitors.len())
            };

            let space = max(space, min_spacing);
            for comp in window.competitors {
                competitors.push(CompetitorWithOffset {
                    competitor: comp,
                    offset: curr_start,
                });
                curr_start = curr_start.add(space);
            }
            if space > spacing_threshold && windows_curr_start + window.duration > curr_start {
                // When space is small/equal to spacing_threshold we don't want to jump to actual window start.
                curr_start = windows_curr_start + window.duration;
            }
        }

        windows_curr_start = window.duration + windows_curr_start;
    }
    competitors
}

fn move_to_prev_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    let popped_competitor = windows[i].competitors.pop_front().unwrap();
    windows[i - 1].competitors.push_back(popped_competitor);

    // stabilize the modified windows
    stabilize_window(windows, i - 1, spacing_threshold);
    stabilize_window(windows, i, spacing_threshold);
}

fn move_to_next_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    let popped_competitor = windows[i].competitors.pop_back().unwrap();
    windows[i + 1].competitors.push_front(popped_competitor);

    // stabilize the modified windows
    stabilize_window(windows, i + 1, spacing_threshold);
    stabilize_window(windows, i, spacing_threshold);
}

fn calculate_window_space(duration: Minutes, competitors_count: usize) -> Minutes {
    if competitors_count == 0 {
        return duration;
    }

    duration.div(competitors_count)
}

fn stabilize_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    if calculate_window_space(windows[i].duration, windows[i].competitors.len())
        <= spacing_threshold
    {
        // curr is passed the spacing threshold, thus competitors movement is possible
        let curr_window_space =
            calculate_window_space(windows[i].duration, windows[i].competitors.len() - 1);
        if i > 0 && i < windows.len() - 1 {
            // Normal window
            let next_window_spacing =
                calculate_window_space(windows[i + 1].duration, windows[i + 1].competitors.len());
            let prev_window_spacing =
                calculate_window_space(windows[i - 1].duration, windows[i - 1].competitors.len());
            if next_window_spacing > curr_window_space && next_window_spacing > prev_window_spacing
            {
                // next is spaced than curr and prev
                move_to_next_window(windows, i, spacing_threshold);
            } else if prev_window_spacing > curr_window_space {
                // prev is spaced than curr and next
                move_to_prev_window(windows, i, spacing_threshold);
            }
        } else if i > 0
            && calculate_window_space(windows[i - 1].duration, windows[i - 1].competitors.len())
                > curr_window_space
        {
            // Last window
            move_to_prev_window(windows, i, spacing_threshold);
        } else if i < windows.len() - 1
            && calculate_window_space(windows[i + 1].duration, windows[i + 1].competitors.len())
                > curr_window_space
        {
            // First window
            move_to_next_window(windows, i, spacing_threshold);
        }
    }
}

fn main() {
    let spacing_threshold = 4;
    let min_spacing = 2;

    let time_windows = vec![
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor { name: "A1".into() },
                Competitor { name: "A2".into() },
                Competitor { name: "A3".into() },
                Competitor { name: "A4".into() },
                Competitor { name: "A5".into() },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::new(),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor { name: "C1".into() },
                Competitor { name: "C2".into() },
                Competitor { name: "C3".into() },
                Competitor { name: "C4".into() },
                Competitor { name: "C5".into() },
                Competitor { name: "C6".into() },
                Competitor { name: "C7".into() },
                Competitor { name: "C8".into() },
                Competitor { name: "C9".into() },
                Competitor { name: "C10".into() },
                Competitor { name: "C11".into() },
                Competitor { name: "C12".into() },
                Competitor { name: "C13".into() },
                Competitor { name: "C14".into() },
                Competitor { name: "C15".into() },
                Competitor { name: "C16".into() },
                Competitor { name: "C17".into() },
                Competitor { name: "C18".into() },
                Competitor { name: "C19".into() },
                Competitor { name: "C20".into() },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor { name: "D1".into() },
                Competitor { name: "D2".into() },
                Competitor { name: "D3".into() },
            ]),
        },
    ];

    let result = generate_startlist(time_windows, spacing_threshold, min_spacing);
    let start_time = chrono::naive::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    for competitor_with_offset in result {
        println!(
            "Competitor: {}, time: {}",
            competitor_with_offset.competitor.name,
            start_time.add(Duration::minutes(competitor_with_offset.offset as i64))
        );
    }
}
