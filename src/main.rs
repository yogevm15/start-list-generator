use std::cmp::max;
use std::ops::{Add, Div};

use chrono::Duration;
use rand::prelude::SliceRandom;
use rand::thread_rng;

struct Competitor {
    name: String,
}

struct CompetitorWithOffset {
    competitor: Competitor,
    offset: Duration,
}

struct Window {
    duration: Duration,
    competitors: Vec<Competitor>,
}

fn generate_startlist(mut windows: Vec<Window>, spacing_threshold: Duration, min_spacing: Duration) -> Vec<CompetitorWithOffset> {
    let mut competitors_count = 0;
    for window in windows.iter_mut() {
        window.competitors.shuffle(&mut thread_rng());
        competitors_count += window.competitors.len();
    }

    for i in 0..windows.len() {
        stabilize_window(&mut windows, i, spacing_threshold);
    }
    let mut competitors = Vec::with_capacity(competitors_count);
    let mut curr_start = Duration::minutes(0);
    let mut windows_curr_start = Duration::minutes(0);
    let windows_len = windows.len();
    for (i, window) in windows.into_iter().enumerate() {
        let mut space = Duration::max_value();
        if window.competitors.len() as i32 != 0 {
            if i == windows_len - 1{
                space = window.duration.div(window.competitors.len() as i32 - 1);
            } else {
                space = window.duration.div(window.competitors.len() as i32);
            }

            space = Duration::minutes(max(space, min_spacing).num_minutes());
            for comp in window.competitors {
                competitors.push(CompetitorWithOffset {
                    competitor: comp,
                    offset: curr_start,
                });
                curr_start = curr_start.add(space);
            }
        }
        if space > spacing_threshold && windows_curr_start + window.duration > curr_start{
            curr_start = windows_curr_start + window.duration;
        }
        windows_curr_start = window.duration + windows_curr_start;
    }
    competitors
}


fn move_to_prev_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Duration){
    let popped_competitor = windows[i].competitors.remove(0);
    windows[i - 1].competitors.push(popped_competitor);
    stabilize_window(windows, i - 1, spacing_threshold);
    stabilize_window(windows, i, spacing_threshold);
}

fn move_to_next_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Duration){
    let popped_competitor = windows[i].competitors.pop().unwrap();
    windows[i + 1].competitors.insert(0, popped_competitor);
    stabilize_window(windows, i + 1, spacing_threshold);
    stabilize_window(windows, i, spacing_threshold);
}

fn calculate_window_space(duration: Duration, competitors_count: usize) -> Duration{
    if competitors_count == 0 {
        return Duration::max_value();
    }

    duration.div(competitors_count as i32)
}

fn stabilize_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Duration) {
    let competitor_count_threshold = windows[i].duration.num_minutes() / spacing_threshold.num_minutes();

    if windows[i].competitors.len() > competitor_count_threshold as usize {
        let curr_window_space = calculate_window_space(windows[i].duration,windows[i].competitors.len() - 1);
        if i > 0 && i < windows.len() - 1{
            let next_window_spacing = calculate_window_space(windows[i + 1].duration, windows[i + 1].competitors.len());
            let prev_window_spacing = calculate_window_space(windows[i - 1].duration,windows[i - 1].competitors.len());
            if next_window_spacing > curr_window_space &&  next_window_spacing >prev_window_spacing{
                move_to_next_window(windows, i , spacing_threshold);
            } else if prev_window_spacing > curr_window_space{
                move_to_prev_window(windows, i , spacing_threshold);
                return;
            }
        } else if i > 0 && calculate_window_space(windows[i - 1].duration,windows[i - 1].competitors.len()) > curr_window_space {
            move_to_prev_window(windows, i , spacing_threshold);
            return;
        } else if i < windows.len() - 1 && calculate_window_space(windows[i + 1].duration, windows[i + 1].competitors.len()) > curr_window_space {
            move_to_next_window(windows, i , spacing_threshold);
        }
    }
}

fn main() {
    let spacing_threshold = Duration::minutes(6); // 5 minutes
    let min_spacing = Duration::minutes(3); // 5 minutes

    let time_windows = vec![
        Window {
            duration: Duration::minutes(30), // 30 minutes
            competitors: vec![
                Competitor {
                    name: "A1".into(),
                },
                Competitor {
                    name: "A2".into(),
                },
                Competitor {
                    name: "A3".into(),
                },
                Competitor {
                    name: "A4".into(),
                },
                Competitor {
                    name: "A5".into(),
                },

            ],
        },
        Window {
            duration: Duration::minutes(30), // 30 minutes
            competitors: vec![

            ],
        },
        Window {
            duration: Duration::minutes(30), // 30 minutes
            competitors: vec![
                Competitor {
                    name: "C1".into(),
                },
                Competitor {
                    name: "C2".into(),
                },
                Competitor {
                    name: "C3".into(),
                },
                Competitor {
                    name: "C4".into(),
                },
                Competitor {
                    name: "C5".into(),
                },
                Competitor {
                    name: "C6".into(),
                },
                Competitor {
                    name: "C7".into(),
                },
                Competitor {
                    name: "C8".into(),
                },
                Competitor {
                    name: "C9".into(),
                },
                Competitor {
                    name: "C10".into(),
                },
                Competitor {
                    name: "C11".into(),
                },
                Competitor {
                    name: "C12".into(),
                },
                Competitor {
                    name: "C13".into(),
                },
                Competitor {
                    name: "C14".into(),
                },
                Competitor {
                    name: "C15".into(),
                },
                Competitor {
                    name: "C16".into(),
                },
                Competitor {
                    name: "C17".into(),
                },
                Competitor {
                    name: "C18".into(),
                },
                Competitor {
                    name: "C19".into(),
                },
                Competitor {
                    name: "C20".into(),
                },
            ],
        },
        Window {
            duration: Duration::minutes(31), // 30 minutes
            competitors: vec![
                Competitor {
                    name: "D1".into(),
                },
                Competitor {
                    name: "D2".into(),
                },
                Competitor {
                    name: "D3".into(),
                },
            ],
        },
    ];

    let result = generate_startlist(time_windows, spacing_threshold,min_spacing);
    let start_time = chrono::naive::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    for competitor_with_offset in result {
        println!(
            "Competitor: {}, time: {}",
            competitor_with_offset.competitor.name,
            start_time.add(competitor_with_offset.offset)
        );
    }
}