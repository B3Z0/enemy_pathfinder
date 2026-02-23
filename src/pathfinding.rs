use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::map::{RuntimeMapAdapter, blocked_for_agent, is_wall, map_height, map_width};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Node {
    f: i32,
    g: i32,
    x: usize,
    y: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.g.cmp(&self.g))
            .then_with(|| other.y.cmp(&self.y))
            .then_with(|| other.x.cmp(&self.x))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn manhattan(a: (usize, usize), b: (usize, usize)) -> i32 {
    let dx = if a.0 > b.0 { a.0 - b.0 } else { b.0 - a.0 };
    let dy = if a.1 > b.1 { a.1 - b.1 } else { b.1 - a.1 };
    (dx + dy) as i32
}

fn neighbors4(x: usize, y: usize) -> [(i32, i32); 4] {
    [
        (x as i32 + 1, y as i32),
        (x as i32 - 1, y as i32),
        (x as i32, y as i32 + 1),
        (x as i32, y as i32 - 1),
    ]
}

pub fn astar(start: (usize, usize), goal: (usize, usize)) -> Vec<(usize, usize)> {
    astar_impl(
        start,
        goal,
        map_width(),
        map_height(),
        |x, y| is_wall(x, y),
        |x, y| blocked_for_agent(x, y),
    )
}

pub fn astar_with_map(
    map: &RuntimeMapAdapter,
    start: (usize, usize),
    goal: (usize, usize),
) -> Vec<(usize, usize)> {
    astar_impl(
        start,
        goal,
        map.width,
        map.height,
        |x, y| map.is_wall(x, y),
        |x, y| map.blocked_for_agent(x, y),
    )
}

fn astar_impl<FIsWall, FBlocked>(
    start: (usize, usize),
    goal: (usize, usize),
    width: usize,
    height: usize,
    is_wall_fn: FIsWall,
    blocked_fn: FBlocked,
) -> Vec<(usize, usize)>
where
    FIsWall: Fn(usize, usize) -> bool,
    FBlocked: Fn(usize, usize) -> bool,
{
    if is_wall_fn(start.0, start.1) || is_wall_fn(goal.0, goal.1) {
        return vec![];
    }
    if start == goal {
        return vec![start];
    }
    if width == 0 || height == 0 {
        return vec![];
    }

    let idx = |x: usize, y: usize| -> usize { y * width + x };

    let mut open = BinaryHeap::<Node>::new();
    let inf: i32 = i32::MAX / 4;

    let cell_count = width * height;
    let mut g_score = vec![inf; cell_count];
    let mut came_from = vec![None::<(usize, usize)>; cell_count];
    let mut closed = vec![false; cell_count];

    g_score[idx(start.0, start.1)] = 0;

    open.push(Node {
        x: start.0,
        y: start.1,
        g: 0,
        f: manhattan(start, goal),
    });

    while let Some(current) = open.pop() {
        let cx = current.x;
        let cy = current.y;
        let current_idx = idx(cx, cy);

        if closed[current_idx] {
            continue;
        }
        closed[current_idx] = true;

        if (cx, cy) == goal {
            let mut path = vec![(cx, cy)];
            let mut cur = (cx, cy);
            while cur != start {
                if let Some(prev) = came_from[idx(cur.0, cur.1)] {
                    cur = prev;
                    path.push(cur);
                } else {
                    return vec![];
                }
            }
            path.reverse();
            return path;
        }

        let current_g = g_score[current_idx];

        for (nx, ny) in neighbors4(cx, cy) {
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= width || ny >= height {
                continue;
            }
            let neighbor_idx = idx(nx, ny);

            if blocked_fn(nx, ny) {
                continue;
            }
            if closed[neighbor_idx] {
                continue;
            }

            let tentative_g = current_g + 1;
            if tentative_g < g_score[neighbor_idx] {
                came_from[neighbor_idx] = Some((cx, cy));
                g_score[neighbor_idx] = tentative_g;

                let h = manhattan((nx, ny), goal);
                open.push(Node {
                    x: nx,
                    y: ny,
                    g: tentative_g,
                    f: tentative_g + h,
                });
            }
        }
    }

    vec![]
}
