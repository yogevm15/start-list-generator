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
    let mut entire_duration = 0;

    for window in windows.iter_mut() {
        window
            .competitors
            .make_contiguous()
            .shuffle(&mut thread_rng());
        competitors_count += window.competitors.len();
        entire_duration += window.duration;
    }
    if competitors_count <= 0 {
        return vec![];
    }

    let entire_spacing = entire_duration.div_ceil(competitors_count);
    let entire_spacing = if entire_spacing <= spacing_threshold {
        // the entire spacing is smaller than spacing_threshold,
        // thus we need don't need to stabilize, just use the entire spacing for all competitors.
        Some(entire_spacing)
    } else {
        for i in 0..windows.len() {
            stabilize_window(&mut windows, i, spacing_threshold);
        }
        None
    };

    let mut competitors = Vec::with_capacity(competitors_count);
    let mut curr_start = 0;
    let mut windows_curr_start = 0;
    for window in windows.into_iter() {
        if window.competitors.len() as i32 != 0 {
            let space = max(
                entire_spacing.unwrap_or(calculate_window_space(
                    window.duration,
                    window.competitors.len(),
                )),
                min_spacing,
            );
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
    for i in 0..windows.len() {
        stabilize_window(windows, i, spacing_threshold);
    }
}

fn move_to_next_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    let popped_competitor = windows[i].competitors.pop_back().unwrap();
    windows[i + 1].competitors.push_front(popped_competitor);

    // stabilize the modified windows
    for i in 0..windows.len() {
        stabilize_window(windows, i, spacing_threshold);
    }
}

fn calculate_window_space(duration: Minutes, competitors_count: usize) -> Minutes {
    if competitors_count == 0 {
        return duration;
    }

    duration.div(competitors_count)
}

fn calculate_window_space_exact(duration: Minutes, competitors_count: usize) -> f64 {
    if competitors_count == 0 {
        return duration as f64;
    }

    (duration as f64).div(competitors_count as f64)
}

fn stabilize_window(windows: &mut Vec<Window>, i: usize, spacing_threshold: Minutes) {
    if calculate_window_space(windows[i].duration, windows[i].competitors.len())
        <= spacing_threshold
    {
        // curr is passed the spacing threshold, thus competitors movement is possible
        let curr_window_space =
            calculate_window_space_exact(windows[i].duration, windows[i].competitors.len() - 1);
        if i > 0 && i < windows.len() - 1 {
            // Normal window
            let next_window_spacing = calculate_window_space_exact(
                windows[i + 1].duration,
                windows[i + 1].competitors.len(),
            );
            let prev_window_spacing = calculate_window_space_exact(
                windows[i - 1].duration,
                windows[i - 1].competitors.len(),
            );
            if next_window_spacing > curr_window_space && next_window_spacing > prev_window_spacing
            {
                // next is spaced than curr and prev
                move_to_next_window(windows, i, spacing_threshold);
            } else if prev_window_spacing > curr_window_space {
                // prev is spaced than curr and next
                move_to_prev_window(windows, i, spacing_threshold);
            }
        } else if i > 0
            && calculate_window_space_exact(
                windows[i - 1].duration,
                windows[i - 1].competitors.len(),
            ) > curr_window_space
        {
            // Last window
            move_to_prev_window(windows, i, spacing_threshold);
        } else if i < windows.len() - 1
            && calculate_window_space_exact(
                windows[i + 1].duration,
                windows[i + 1].competitors.len(),
            ) > curr_window_space
        {
            // First window
            move_to_next_window(windows, i, spacing_threshold);
        }
    }
}

fn main() {
    let spacing_threshold = 3;
    let min_spacing = 2;

    let mut time_windows = vec![
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor {
                    name: String::from("Levit Kristina"),
                },
                Competitor {
                    name: String::from("טייטר מקסים"),
                },
                Competitor {
                    name: String::from("צינדר אריאל"),
                },
                Competitor {
                    name: String::from("שפר שי"),
                },
                Competitor {
                    name: String::from("UKR Ersteniuk Volodymyr"),
                },
                Competitor {
                    name: String::from("ליזוגוב ויאצ'סלב"),
                },
                Competitor {
                    name: String::from("שחורי דניאל"),
                },
                Competitor {
                    name: String::from("שפר שחר"),
                },
                Competitor {
                    name: String::from("אורי אופיר"),
                },
                Competitor {
                    name: String::from("גמינדר תומר"),
                },
                Competitor {
                    name: String::from("ויינר תומר"),
                },
                Competitor {
                    name: String::from("לדרר חגי"),
                },
                Competitor {
                    name: String::from("מלץ אבי"),
                },
                Competitor {
                    name: String::from("רגבי לירון"),
                },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor {
                    name: String::from("קלקשטיין בר"),
                },
                Competitor {
                    name: String::from("קלקשטיין יובל"),
                },
                Competitor {
                    name: String::from("שחורי אלון"),
                },
                Competitor {
                    name: String::from("ילין יואב"),
                },
                Competitor {
                    name: String::from("לרמן שגיא"),
                },
                Competitor {
                    name: String::from("לשצ'נקו ניקיטה"),
                },
                Competitor {
                    name: String::from("נוסבוים איתם"),
                },
                Competitor {
                    name: String::from("קלקשטיין נדב"),
                },
                Competitor {
                    name: String::from("רז-רוטשילד דניאל"),
                },
                Competitor {
                    name: String::from("ריינליב דינסטי"),
                },
                Competitor {
                    name: String::from("שפירא אורן"),
                },
                Competitor {
                    name: String::from("אשכנזי אסף"),
                },
                Competitor {
                    name: String::from("גלוזשטיין ולרי"),
                },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([
                Competitor {
                    name: String::from("מאירקוביץ מארינה"),
                },
                Competitor {
                    name: String::from("מצפון גאיה"),
                },
                Competitor {
                    name: String::from("אילין רומן"),
                },
                Competitor {
                    name: String::from("מאירקוביץ יבגני"),
                },
                Competitor {
                    name: String::from("רון ירדן"),
                },
                Competitor {
                    name: String::from("שפיצר בן ארי"),
                },
                Competitor {
                    name: String::from("קורוטייב מיכאיל"),
                },
            ]),
        },
        Window {
            duration: 30, // 30 minutes
            competitors: VecDeque::from([]),
        },
    ];
    time_windows.reverse();
    let result = generate_startlist(time_windows, spacing_threshold, min_spacing);
    let start_time = chrono::naive::NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    for (i, competitor_with_offset) in result.iter().enumerate() {
        println!(
            "[{}] Competitor: {}, time: {}",
            i + 1,
            competitor_with_offset.competitor.name,
            start_time.add(Duration::minutes(competitor_with_offset.offset as i64))
        );
    }
}
