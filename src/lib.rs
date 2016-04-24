#![allow(warnings)] 

extern crate bit_vec;

use std::iter::FromIterator;

use bit_vec::BitVec;

#[derive(PartialEq, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x: x, y: y }
    }

    fn sq_seg_dist(&self, p1: &Point, p2: &Point) -> f64 {
        let mut x = p1.x;
        let mut y = p1.y;
        let mut dx = p2.x - p1.x;
        let mut dy = p2.y - p1.y;

        if dx != 0.0 || dy != 0.0 {
            let t = ((self.x - p1.x) * dx + (self.y - p1.y) * dy) / (dx * dx + dy * dy);

            if t > 1.0 {
                x = p2.x;
                y = p2.y;
            } else if t > 0.0 {
                x += dx * t;
                y += dy * t;
            }
        }

        dx = self.x - x;
        dy = self.y - y;

        (dx * dx + dy * dy)
    }
}

#[derive(PartialEq, Clone)]
pub struct Polyline {
    pub points: Vec<Point>
}

impl FromIterator<Point> for Polyline {
    fn from_iter<I: IntoIterator<Item=Point>>(iterator: I) -> Self {
        let mut polyline = Polyline::new();
        for i in iterator {
            polyline.points.push(i);
        }
        polyline
    }
}

impl Polyline {
    pub fn new() -> Self {
        Polyline { points: Vec::new() }
    }

	pub fn from_vec(vec: Vec<Point>) -> Self {
        Polyline { points: vec }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    fn add(&mut self, point: Point) {
        self.points.push(point);
    }

    fn simplify_radial_dist(&self, sq_tolerance: f64) -> Self {
        self.points.iter().take(self.points.len()-1)
            .zip(self.points.iter().skip(1))
            .filter(|&(pre, cur)| {
                let dx = pre.x - cur.y;
                let dy = pre.y - cur.y;
                (dx * dx + dy * dy) > sq_tolerance
            })
            .map(|(pre, cur)| pre.clone())
            .collect()
    }

    fn simplify_douglas_peucker(&self, sq_tolerance: f64) -> Self {
        let mut stack: Vec<(usize, usize)> = Vec::new();
        stack.push((0, self.points.len() - 1));

        let mut keep_elem_vec = BitVec::from_elem(self.points.len(), true);


        while !stack.is_empty() {
            let (start_idx, end_idx) = stack.pop().unwrap();

            let mut dmax: f64 = 0.0f64;
            let mut max_idx: usize = start_idx;

            for i in (start_idx + 1)..end_idx {
                if keep_elem_vec.get(i) == Some(true) {
                    let seg_dist = self.points.get(i).unwrap()
                        .sq_seg_dist(
                            self.points.get(start_idx).unwrap(),
                            self.points.get(end_idx).unwrap());
                    if seg_dist > dmax {
                        max_idx = i;
                        dmax = seg_dist;
                    }
                }
            }

            if dmax > sq_tolerance {
                stack.push((start_idx, max_idx));
                stack.push((max_idx, end_idx));
            } else {
                for i in (start_idx + 1)..end_idx {
                    keep_elem_vec.set(i, false);
                }
            }
        }

        self.points.iter()
            .enumerate()
            .filter(|&(i, p)| keep_elem_vec.get(i) == Some(true))
            .map(|(i, p)| p.clone())
            .collect()
    }

    pub fn simplify(&self, tolerance: f64, highest_quality: bool) -> Polyline {
        if self.points.len() <= 2 {
            return self.clone();
        }

        let sq_tolerance = tolerance.powi(2);

        // TODO(talevy): figure out right thing to do here
        // skipping this value for now since it doesn't seem to help
        let poly = if highest_quality {
            self.clone()
        } else {
            self.simplify_radial_dist(sq_tolerance)
        }.simplify_douglas_peucker(sq_tolerance);

        // TODO(talevy): port this simplification algorithm 
        // out into its own method
        let mut keep = Vec::with_capacity(self.points.len());
        let mut it = self.points.iter();
        let mut q = it.next().unwrap();
        keep.push(q.clone());

        for p in it {
            let dx = p.x - q.x;
            let dy = p.y - q.y;
            let d = (dx * dx + dy * dy);

            if d > 0.000009 {
                keep.push(p.clone());
                q = p;
            }
        }

        Polyline::from_vec(keep)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    impl fmt::Debug for Point {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "({},{})", self.x, self.y);
            Ok(())
        }
    }
    impl fmt::Debug for Polyline {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "[");
            for (i, p) in self.points.iter().enumerate() {
                write!(fmt, "{:?}", p);
                if i < self.points.len() - 1 {
                    write!(fmt, ",");
                }
            }
            write!(fmt, "]");
            Ok(())
        }
    }

    #[test]
    fn does_nothing_with_two() {
        let mut line = Polyline::new();
        line.add(Point::new(0.0, 0.0));
        line.add(Point::new(1.0, 8.9));
        let new = line.simplify(5.0, true);
        assert_eq!("[(0,0),(1,8.9)]", format!("{:?}", new));
    }

    #[test]
    fn it_works() {
		let original = Polyline::from_vec(vec![
			Point {x:224.55,y:250.15},Point {x:226.91,y:244.19},Point {x:233.31,y:241.45},Point {x:234.98,y:236.06},
			Point {x:244.21,y:232.76},Point {x:262.59,y:215.31},Point {x:267.76,y:213.81},Point {x:273.57,y:201.84},
			Point {x:273.12,y:192.16},Point {x:277.62,y:189.03},Point {x:280.36,y:181.41},Point {x:286.51,y:177.74},
			Point {x:292.41,y:159.37},Point {x:296.91,y:155.64},Point {x:314.95,y:151.37},Point {x:319.75,y:145.16},
			Point {x:330.33,y:137.57},Point {x:341.48,y:139.96},Point {x:369.98,y:137.89},Point {x:387.39,y:142.51},
			Point {x:391.28,y:139.39},Point {x:409.52,y:141.14},Point {x:414.82,y:139.75},Point {x:427.72,y:127.30},
			Point {x:439.60,y:119.74},Point {x:474.93,y:107.87},Point {x:486.51,y:106.75},Point {x:489.20,y:109.45},
			Point {x:493.79,y:108.63},Point {x:504.74,y:119.66},Point {x:512.96,y:122.35},Point {x:518.63,y:120.89},
			Point {x:524.09,y:126.88},Point {x:529.57,y:127.86},Point {x:534.21,y:140.93},Point {x:539.27,y:147.24},
			Point {x:567.69,y:148.91},Point {x:575.25,y:157.26},Point {x:580.62,y:158.15},Point {x:601.53,y:156.85},
			Point {x:617.74,y:159.86},Point {x:622.00,y:167.04},Point {x:629.55,y:194.60},Point {x:638.90,y:195.61},
			Point {x:641.26,y:200.81},Point {x:651.77,y:204.56},Point {x:671.55,y:222.55},Point {x:683.68,y:217.45},
			Point {x:695.25,y:219.15},Point {x:700.64,y:217.98},Point {x:703.12,y:214.36},Point {x:712.26,y:215.87},
			Point {x:721.49,y:212.81},Point {x:727.81,y:213.36},Point {x:729.98,y:208.73},Point {x:735.32,y:208.20},
			Point {x:739.94,y:204.77},Point {x:769.98,y:208.42},Point {x:779.60,y:216.87},Point {x:784.20,y:218.16},
			Point {x:800.24,y:214.62},Point {x:810.53,y:219.73},Point {x:817.19,y:226.82},Point {x:820.77,y:236.17},
			Point {x:827.23,y:236.16},Point {x:829.89,y:239.89},Point {x:851.00,y:248.94},Point {x:859.88,y:255.49},
			Point {x:865.21,y:268.53},Point {x:857.95,y:280.30},Point {x:865.48,y:291.45},Point {x:866.81,y:298.66},
			Point {x:864.68,y:302.71},Point {x:867.79,y:306.17},Point {x:859.87,y:311.37},Point {x:860.08,y:314.35},
			Point {x:858.29,y:314.94},Point {x:858.10,y:327.60},Point {x:854.54,y:335.40},Point {x:860.92,y:343.00},
			Point {x:856.43,y:350.15},Point {x:851.42,y:352.96},Point {x:849.84,y:359.59},Point {x:854.56,y:365.53},
			Point {x:849.74,y:370.38},Point {x:844.09,y:371.89},Point {x:844.75,y:380.44},Point {x:841.52,y:383.67},
			Point {x:839.57,y:390.40},Point {x:845.59,y:399.05},Point {x:848.40,y:407.55},Point {x:843.71,y:411.30},
			Point {x:844.09,y:419.88},Point {x:839.51,y:432.76},Point {x:841.33,y:441.04},Point {x:847.62,y:449.22},
			Point {x:847.16,y:458.44},Point {x:851.38,y:462.79},Point {x:853.97,y:471.15},Point {x:866.36,y:480.77}
		]);

        let expected = Polyline::from_vec(vec![
            Point {x:224.55,y:250.15},Point {x:267.76,y:213.81},Point {x:296.91,y:155.64},Point {x:330.33,y:137.57},
            Point {x:409.52,y:141.14},Point {x:439.60,y:119.74},Point {x:486.51,y:106.75},Point {x:529.57,y:127.86},
            Point {x:539.27,y:147.24},Point {x:617.74,y:159.86},Point {x:629.55,y:194.60},Point {x:671.55,y:222.55},
            Point {x:727.81,y:213.36},Point {x:739.94,y:204.77},Point {x:769.98,y:208.42},Point {x:779.60,y:216.87},
            Point {x:800.24,y:214.62},Point {x:820.77,y:236.17},Point {x:859.88,y:255.49},Point {x:865.21,y:268.53},
            Point {x:857.95,y:280.30},Point {x:867.79,y:306.17},Point {x:859.87,y:311.37},Point {x:854.54,y:335.40},
            Point {x:860.92,y:343.00},Point {x:849.84,y:359.59},Point {x:854.56,y:365.53},Point {x:844.09,y:371.89},
            Point {x:839.57,y:390.40},Point {x:848.40,y:407.55},Point {x:839.51,y:432.76},Point {x:853.97,y:471.15},
            Point {x:866.36,y:480.77}
        ]);

        let actual = original.simplify(5.0, false);

        assert_eq!(actual, expected);
    }
}
