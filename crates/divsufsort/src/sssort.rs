use crate::{common::*, crosscheck, crosscheck::*};
use std::{cmp, default::Default, mem};

//--------------------
// Private functions
//--------------------

#[rustfmt::skip]
const lg_table: [Idx; 256] = [
 -1,0,1,1,2,2,2,2,3,3,3,3,3,3,3,3,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,
  5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,
  6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,
  6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7
];

/// Fast log2, using lookup tables
#[inline(always)]
pub fn ss_ilg<N: Into<Idx>>(n: N) -> Idx {
    let n: Idx = n.into();

    if (n & 0xff00) > 0 {
        8 + lg_table[((n >> 8) & 0xff) as usize]
    } else {
        0 + lg_table[((n >> 0) & 0xff) as usize]
    }
}

#[rustfmt::skip]
const sqq_table: [Idx; 256] = [
  0,  16,  22,  27,  32,  35,  39,  42,  45,  48,  50,  53,  55,  57,  59,  61,
 64,  65,  67,  69,  71,  73,  75,  76,  78,  80,  81,  83,  84,  86,  87,  89,
 90,  91,  93,  94,  96,  97,  98,  99, 101, 102, 103, 104, 106, 107, 108, 109,
110, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126,
128, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142,
143, 144, 144, 145, 146, 147, 148, 149, 150, 150, 151, 152, 153, 154, 155, 155,
156, 157, 158, 159, 160, 160, 161, 162, 163, 163, 164, 165, 166, 167, 167, 168,
169, 170, 170, 171, 172, 173, 173, 174, 175, 176, 176, 177, 178, 178, 179, 180,
181, 181, 182, 183, 183, 184, 185, 185, 186, 187, 187, 188, 189, 189, 190, 191,
192, 192, 193, 193, 194, 195, 195, 196, 197, 197, 198, 199, 199, 200, 201, 201,
202, 203, 203, 204, 204, 205, 206, 206, 207, 208, 208, 209, 209, 210, 211, 211,
212, 212, 213, 214, 214, 215, 215, 216, 217, 217, 218, 218, 219, 219, 220, 221,
221, 222, 222, 223, 224, 224, 225, 225, 226, 226, 227, 227, 228, 229, 229, 230,
230, 231, 231, 232, 232, 233, 234, 234, 235, 235, 236, 236, 237, 237, 238, 238,
239, 240, 240, 241, 241, 242, 242, 243, 243, 244, 244, 245, 245, 246, 246, 247,
247, 248, 248, 249, 249, 250, 250, 251, 251, 252, 252, 253, 253, 254, 254, 255
];

/// Fast sqrt, using lookup tables
#[allow(overflowing_literals)] // ☠☠☠
#[inline(always)]
pub fn ss_isqrt<X: Into<Idx>>(x: X) -> Idx {
    let x: Idx = x.into();

    let mut y: Idx;
    let e: Idx;

    if x >= (SS_BLOCKSIZE * SS_BLOCKSIZE) {
        return SS_BLOCKSIZE;
    }

    e = if (x & 0xffff_0000) > 0 {
        if (x & 0xff00_0000) > 0 {
            24 + lg_table[((x >> 24) & 0xff) as usize]
        } else {
            16 + lg_table[((x >> 16) & 0xff) as usize]
        }
    } else {
        if (x & 0x0000_ff00) > 0 {
            8 + lg_table[(((x >> 8) & 0xff) as usize)]
        } else {
            0 + lg_table[(((x >> 0) & 0xff) as usize)]
        }
    };

    if e >= 16 {
        y = sqq_table[(x >> ((e - 6) - (e & 1))) as usize] << ((e >> 1) - 7);
        if e >= 24 {
            y = (y + 1 + x / y) >> 1;
        }
        y = (y + 1 + x / y) >> 1;
    } else if e >= 8 {
        y = (sqq_table[(x >> ((e - 6) - (e & 1))) as usize] >> (7 - (e >> 1))) + 1;
    } else {
        return sqq_table[x as usize] >> 4;
    }

    if x < (y * y) {
        y - 1
    } else {
        y
    }
}

//------------------------------------------------------------------------------

/// Compare two suffixes
#[inline(always)]
pub fn ss_compare(
    T: &Text,
    SAp1: &SuffixArray,
    p1: SAPtr,
    SAp2: &SuffixArray,
    p2: SAPtr,
    depth: Idx,
) -> Idx {
    let mut U1 = depth + SAp1[p1];
    let mut U2 = depth + SAp2[p2];
    let U1n = SAp1[p1 + 1] + 2;
    let U2n = SAp2[p2 + 1] + 2;

    while (U1 < U1n) && (U2 < U2n) && (T[U1] == T[U2]) {
        U1 += 1;
        U2 += 1;
    }

    let res = if U1 < U1n {
        if U2 < U2n {
            T.get(U1) - T.get(U2)
        } else {
            1
        }
    } else {
        if U2 < U2n {
            -1
        } else {
            0
        }
    };
    res
}

//------------------------------------------------------------------------------

/// Insertionsort for small size groups
pub fn ss_insertionsort(
    T: &Text,
    SA: &mut SuffixArray,
    PA: SAPtr,
    first: SAPtr,
    last: SAPtr,
    depth: Idx,
) {
    let mut i: SAPtr;
    let mut j: SAPtr;
    let mut t: Idx;
    let mut r: Idx;

    i = last - 2;
    // for 1
    while first <= i {
        t = SA[i];
        j = i + 1;

        // for 2
        loop {
            // cond for 2
            r = ss_compare(T, SA, PA + t, SA, PA + SA[j], depth);
            if !(0 < r) {
                break;
            }

            // body for 2

            // do while
            loop {
                SA[j - 1] = SA[j];

                j += 1;
                if !((j < last) && SA[j] < 0) {
                    break;
                }
            }

            if (last <= j) {
                break;
            }

            // iter for 2 (empty)
        }

        if (r == 0) {
            SA[j] = !SA[j];
        }
        SA[j - 1] = t;

        // iter
        i -= 1;
    }
}

//------------------------------------------------------------------------------

// TODO: document?
pub fn ss_fixdown(
    T: &Text,
    Td: Idx,
    PA: SAPtr,
    SA_top: &mut SuffixArray,
    first: SAPtr,
    mut i: Idx,
    size: Idx,
) {
    let mut j: Idx;
    let mut v: Idx;
    let mut c: Idx;
    let mut d: Idx;
    let mut e: Idx;
    let mut k: Idx;

    macro_rules! Td {
        ($x: expr) => {
            T.get(Td + $x)
        };
    }
    macro_rules! PA {
        ($x: expr) => {
            SA_top[PA + $x]
        };
    }
    macro_rules! SA {
        ($x: expr) => {
            SA_top[first + $x]
        };
    }

    v = SA!(i);
    c = Td!(PA!(v));

    // BEAST
    loop {
        // cond
        j = 2 * i + 1;
        if !(j < size) {
            break;
        }
        crosscheck!("BEAST");

        // body
        k = j;
        j += 1;

        d = Td!(PA!(SA!(k)));
        e = Td!(PA!(SA!(j)));
        if (d < e) {
            crosscheck!("d<e");
            k = j;
            d = e;
        }
        if (d <= c) {
            crosscheck!("d<=c");
            break;
        }

        // iter
        SA!(i) = SA!(k);
        i = k;
    }
    SA!(i) = v;
}

/// Simple top-down heapsort.
pub fn ss_heapsort(
    T: &Text,
    Td: Idx,
    SA_top: &mut SuffixArray,
    PA: SAPtr,
    first: SAPtr,
    size: Idx,
) {
    let mut i: Idx;
    let mut m = size;
    let mut t: Idx;

    macro_rules! Td {
        ($x: expr) => {
            T[Td + $x]
        };
    };
    macro_rules! PA {
        ($x: expr) => {
            SA_top[PA + $x]
        };
    };
    macro_rules! SA {
        ($x: expr) => {
            SA_top[first + $x]
        };
    }
    macro_rules! SA_swap {
        ($x: expr, $y: expr) => {
            SA_top.swap($x + first, $y + first)
        };
    }

    if (size % 2) == 0 {
        m -= 1;
        if Td!(PA!(SA!(m / 2))) < Td!(PA!(SA!(m))) {
            crosscheck!("SWAP {} {}", m, m / 2);
            SA_swap!(SAPtr(m), SAPtr(m / 2));
        }
    }

    // LADY
    for i in (0..(m / 2)).rev() {
        crosscheck!("LADY {}", i);
        ss_fixdown(T, Td, PA, SA_top, first, i, m);
    }

    if (size % 2) == 0 {
        SA_swap!(SAPtr(0), SAPtr(m));
        ss_fixdown(T, Td, PA, SA_top, first, 0, m);
    }

    // TRUMPET
    for i in (1..m).rev() {
        crosscheck!("TRUMPET {}", i);
        t = SA!(0);
        SA!(0) = SA!(i);
        ss_fixdown(T, Td, PA, SA_top, first, 0, i);
        SA!(i) = t;
    }
}

//------------------------------------------------------------------------------

/// Returns the median of three elements
#[inline(always)]
pub fn ss_median3(
    T: &Text,
    Td: Idx,
    SA: &SuffixArray,
    PA: SAPtr,
    mut v1: SAPtr,
    mut v2: SAPtr,
    v3: SAPtr,
) -> SAPtr {
    let mut t: SAPtr;
    macro_rules! get {
        ($x: expr) => {
            T[Td + SA[PA + SA[$x]]]
        };
    }

    if get!(v1) > get!(v2) {
        mem::swap(&mut v1, &mut v2);
    }
    if get!(v2) > get!(v3) {
        if get!(v1) > get!(v3) {
            v1
        } else {
            v3
        }
    } else {
        v2
    }
}

/// Returns the median of five elements
#[inline(always)]
pub fn ss_median5(
    T: &Text,
    Td: Idx,
    SA: &SuffixArray,
    PA: SAPtr,
    mut v1: SAPtr,
    mut v2: SAPtr,
    mut v3: SAPtr,
    mut v4: SAPtr,
    mut v5: SAPtr,
) -> SAPtr {
    let mut t: SAPtr;
    macro_rules! get {
        ($x: expr) => {
            T[Td + SA[PA + SA[$x]]]
        };
    }
    if get!(v2) > get!(v3) {
        mem::swap(&mut v2, &mut v3);
    }
    if get!(v4) > get!(v5) {
        mem::swap(&mut v4, &mut v5);
    }
    if get!(v2) > get!(v4) {
        mem::swap(&mut v2, &mut v4);
        mem::swap(&mut v3, &mut v5);
    }
    if get!(v1) > get!(v3) {
        mem::swap(&mut v1, &mut v3);
    }
    if get!(v1) > get!(v4) {
        mem::swap(&mut v1, &mut v4);
        mem::swap(&mut v3, &mut v5);
    }
    if get!(v3) > get!(v4) {
        v4
    } else {
        v3
    }
}

/// Returns the pivot element
#[inline(always)]
pub fn ss_pivot(
    T: &Text,
    Td: Idx,
    SA: &SuffixArray,
    PA: SAPtr,
    mut first: SAPtr,
    mut last: SAPtr,
) -> SAPtr {
    let mut t: Idx = (last - first).0;
    let mut middle: SAPtr = first + (t / 2);

    if t <= 512 {
        if t <= 32 {
            return ss_median3(T, Td, SA, PA, first, middle, last - 1);
        } else {
            t >>= 2;
            return ss_median5(
                T,
                Td,
                SA,
                PA,
                first,
                first + t,
                middle,
                last - 1 - t,
                last - 1,
            );
        }
    }
    t >>= 3;
    first = ss_median3(T, Td, SA, PA, first, first + t, first + (t << 1));
    middle = ss_median3(T, Td, SA, PA, middle - t, middle, middle + t);
    last = ss_median3(T, Td, SA, PA, last - 1 - (t << 1), last - 1 - t, last - 1);
    ss_median3(T, Td, SA, PA, first, middle, last)
}

//------------------------------------------------------------------------------

/// Binary partition for substrings.
#[inline(always)]
pub fn ss_partition(
    SA: &mut SuffixArray,
    PA: SAPtr,
    first: SAPtr,
    last: SAPtr,
    depth: Idx,
) -> SAPtr {
    macro_rules! PA {
        ($x: expr) => {
            SA[PA + $x]
        };
    }
    // JIMMY
    let mut a = first - 1;
    let mut b = last;
    macro_rules! a {
        () => {
            SA[a]
        };
    }
    macro_rules! b {
        () => {
            SA[b]
        };
    }

    loop {
        // JANINE
        loop {
            a += 1;
            if !(a < b) {
                break;
            }
            if !((PA!(a!()) + depth) >= (PA!(a!() + 1) + 1)) {
                break;
            }

            // loop body
            a!() = !a!();
        }

        // GEORGIO
        loop {
            b -= 1;
            if !(a < b) {
                break;
            }
            if !((PA!(b!()) + depth) < (PA!(b!() + 1) + 1)) {
                break;
            }

            // loop body is empty
        }

        if b <= a {
            break;
        }

        let t = !b!();
        b!() = a!();
        a!() = t;
    }

    if (first < a) {
        SA[first] = !SA[first];
    }
    a
}

/// Multikey introsort for medium size groups
pub fn ss_mintrosort(
    T: &Text,
    SA: &mut SuffixArray,
    PA: SAPtr,
    mut first: SAPtr,
    mut last: SAPtr,
    mut depth: Idx,
) {
    macro_rules! PA {
        ($x: expr) => {
            SA[PA + $x]
        };
    };

    let mut stack = Stack::new();

    let mut a: SAPtr;
    let mut b: SAPtr;
    let mut c: SAPtr;
    let mut d: SAPtr;
    let mut e: SAPtr;
    let mut f: SAPtr;

    let mut s: Idx;
    let mut t: Idx;

    let mut limit: Idx;
    let mut v: Idx;
    let mut x: Idx = 0;

    limit = ss_ilg(last - first);

    loop {
        if ((last - first) <= SS_INSERTIONSORT_THRESHOLD) {
            if (1 < (last - first)) {
                ss_insertionsort(T, SA, PA, first, last, depth);
            }
            if !stack
                .pop(&mut first, &mut last, &mut depth, &mut limit)
                .is_ok()
            {
                return;
            }
            continue;
        }

        let Td = depth;
        macro_rules! Td {
            ($x: expr) => {
                T.get(Td + $x)
            };
        }
        macro_rules! TdPAStar {
            ($x: expr) => {
                Td!(PA!(SA[$x]))
            };
        }

        if (limit == 0) {
            SA_dump(&SA.range(first..last), "before heapsort");
            ss_heapsort(T, Td, SA, PA, first, (last - first).into());
            SA_dump(&SA.range(first..last), "after heapsort");
        }
        limit -= 1;

        if (limit < 0) {
            crosscheck!("limit < 0");
            a = first + 1;
            v = TdPAStar!(first);

            // DAVE
            while a < last {
                crosscheck!("DAVE");
                x = TdPAStar!(a);
                if (x != v) {
                    if (1 < (a - first)) {
                        break;
                    }
                    v = x;
                    first = a;
                }

                // loop iter
                a += 1;
            }

            if TdPAStar!(first) < v {
                crosscheck!("call ss_partition");
                crosscheck!("first(A)={}", first - PA);
                first = ss_partition(SA, PA, first, a, depth);
                crosscheck!("first(B)={}", first - PA);
            }

            if (a - first) <= (last - a) {
                if 1 < (a - first) {
                    stack.push(a, last, depth, -1);
                    last = a;
                    depth += 1;
                    limit = ss_ilg(a - first);
                } else {
                    first = a;
                    limit -= 1;
                }
            } else {
                if 1 < (last - a) {
                    stack.push(first, a, depth + 1, ss_ilg(a - first));
                    first = a;
                    limit = -1;
                } else {
                    last = a;
                    depth += 1;
                    limit = ss_ilg(a - first);
                }
            }
            continue;
        }

        // choose pivot
        a = ss_pivot(T, Td, SA, PA, first, last);
        v = TdPAStar!(a);
        SA.swap(first, a);

        // partition
        // NORA
        b = first;
        loop {
            b += 1;
            if !(b < last) {
                break;
            }
            x = TdPAStar!(b);
            if !(x == v) {
                break;
            }
            // body
            crosscheck!("NORA");
        }
        a = b;
        if (a < last) && (x < v) {
            // STAN
            loop {
                b += 1;
                if !(b < last) {
                    break;
                }
                x = TdPAStar!(b);
                if !(x <= v) {
                    break;
                }
                // body
                crosscheck!("STAN");
                if x == v {
                    SA.swap(b, a);
                    a += 1;
                }
            }
        }

        // NATHAN
        c = last;
        loop {
            c -= 1;
            if !(b < c) {
                break;
            }
            x = TdPAStar!(c);
            if !(x == v) {
                break;
            }
            // body
            crosscheck!("NATHAN");
        }
        d = c;
        if (b < d) && (x > v) {
            // JACOB
            loop {
                c -= 1;
                if !(b < c) {
                    break;
                }
                x = TdPAStar!(c);
                if !(x >= v) {
                    break;
                }
                // body
                crosscheck!("JACOB");
                if x == v {
                    SA.swap(c, d);
                    d -= 1;
                }
            }
        }

        // RITA
        while b < c {
            crosscheck!("RITA");
            SA.swap(b, c);
            // ROMEO
            loop {
                b += 1;
                if !(b < c) {
                    break;
                }
                x = TdPAStar!(b);
                if !(x <= v) {
                    break;
                }
                // body
                crosscheck!("ROMEO");
                if x == v {
                    SA.swap(b, a);
                    a += 1;
                }
            }
            // JULIET
            loop {
                c -= 1;
                if !(b < c) {
                    break;
                }
                x = TdPAStar!(c);
                if !(x >= v) {
                    break;
                }
                // body
                crosscheck!("JULIET");
                if x == v {
                    SA.swap(c, d);
                    d -= 1;
                }
            }
        }

        if a <= d {
            c = b - 1;
            s = (a - first).0;
            t = (b - a).0;
            if s > t {
                s = t;
            }

            // JOSHUA
            e = first;
            f = b - s;
            while 0 < s {
                crosscheck!("JOSHUA");
                SA.swap(e, f);
                s -= 1;
                e += 1;
                f += 1;
            }
            s = (d - c).0;
            t = (last - d - 1).0;
            if s > t {
                s = t;
            }
            // BERENICE
            e = b;
            f = last - s;
            while 0 < s {
                crosscheck!("BERENICE");
                SA.swap(e, f);
                s -= 1;
                e += 1;
                f += 1;
            }

            a = first + (b - a);
            c = last - (d - c);
            b = if v <= Td!(PA!(SA[a]) - 1) {
                a
            } else {
                let res = ss_partition(SA, PA, a, c, depth);
                res
            };

            if (a - first) <= (last - c) {
                if (last - c) <= (c - b) {
                    stack.push(b, c, depth + 1, ss_ilg(c - b));
                    stack.push(c, last, depth, limit);
                    last = a;
                } else if (a - first) <= (c - b) {
                    stack.push(c, last, depth, limit);
                    stack.push(b, c, depth + 1, ss_ilg(c - b));
                    last = a;
                } else {
                    stack.push(c, last, depth, limit);
                    stack.push(first, a, depth, limit);
                    first = b;
                    last = c;
                    depth += 1;
                    limit = ss_ilg(c - b);
                }
            } else {
                if (a - first) <= (c - b) {
                    stack.push(b, c, depth + 1, ss_ilg(c - b));
                    stack.push(first, a, depth, limit);
                    first = c;
                } else if (last - c) <= (c - b) {
                    stack.push(first, a, depth, limit);
                    stack.push(b, c, depth + 1, ss_ilg(c - b));
                    first = c;
                } else {
                    stack.push(first, a, depth, limit);
                    stack.push(c, last, depth, limit);
                    first = b;
                    last = c;
                    depth += 1;
                    limit = ss_ilg(c - b);
                }
            }
        } else {
            limit += 1;
            if (TdPAStar!(first) - 1) < v {
                first = ss_partition(SA, PA, first, last, depth);
                limit = ss_ilg(last - first);
            }
            depth += 1;
        }
    }
}

//------------------------------------------------------------------------------

#[inline(always)]
pub fn ss_blockswap(SA: &mut SuffixArray, a: SAPtr, b: SAPtr, n: Idx) {
    unimplemented!()
}

#[inline(always)]
pub fn ss_rotate(SA: &mut SuffixArray, first: SAPtr, middle: SAPtr, last: SAPtr) {
    unimplemented!()
}

//------------------------------------------------------------------------------

pub fn ss_inplacemerge(
    T: &Text,
    SA: &SuffixArray,
    PA: SAPtr,
    first: SAPtr,
    middle: SAPtr,
    last: SAPtr,
    depth: Idx,
) {
    unimplemented!()
}

//------------------------------------------------------------------------------

/// Merge-forward with internal buffer
pub fn ss_mergeforward(
    T: &Text,
    SA: &mut SuffixArray,
    PA: SAPtr,
    first: SAPtr,
    middle: SAPtr,
    last: SAPtr,
    buf: SAPtr,
    depth: Idx,
) {
    unimplemented!()
}

/// Merge-backward with internal buffer
pub fn ss_mergebackward(
    T: &Text,
    SA: &mut SuffixArray,
    PA: SAPtr,
    first: SAPtr,
    middle: SAPtr,
    last: SAPtr,
    buf: SAPtr,
    depth: Idx,
) {
    unimplemented!()
}

const MERGE_STACK_SIZE: usize = 32;

struct MergeStackItem {
    a: SAPtr,
    b: SAPtr,
    c: SAPtr,
    d: Idx,
}

impl Default for MergeStackItem {
    fn default() -> Self {
        Self {
            a: SAPtr(0),
            b: SAPtr(0),
            c: SAPtr(0),
            d: 0,
        }
    }
}

struct MergeStack {
    items: [MergeStackItem; MERGE_STACK_SIZE],
    size: usize,
}

impl MergeStack {
    fn new() -> Self {
        Self {
            items: Default::default(),
            size: 0,
        }
    }

    #[inline(always)]
    fn push(&mut self, a: SAPtr, b: SAPtr, c: SAPtr, d: Idx) {
        assert!(self.size < STACK_SIZE);
        self.items[self.size].a = a;
        self.items[self.size].b = b;
        self.items[self.size].c = c;
        self.items[self.size].d = d;
        self.size += 1;
    }

    #[inline(always)]
    #[must_use]
    fn pop(&mut self, a: &mut SAPtr, b: &mut SAPtr, c: &mut SAPtr, d: &mut Idx) -> Result<(), ()> {
        if (self.size == 0) {
            Err(())
        } else {
            self.size -= 1;
            *a = self.items[self.size].a;
            *b = self.items[self.size].b;
            *c = self.items[self.size].c;
            *d = self.items[self.size].d;
            Ok(())
        }
    }
}

/// D&C based merge
pub fn ss_swapmerge(
    T: &Text,
    SA: &mut SuffixArray,
    PA: SAPtr,
    mut first: SAPtr,
    mut middle: SAPtr,
    mut last: SAPtr,
    buf: SAPtr,
    bufsize: Idx,
    depth: Idx,
) {
    macro_rules! get_idx {
        ($a: expr) => {
            if 0 <= $a {
                $a
            } else {
                !$a
            }
        };
    }
    macro_rules! merge_check {
        ($a: expr, $b: expr, $c: expr) => {
            if ($c & 1 > 0)
                || (($c & 2 > 0)
                    && (ss_compare(T, SA, PA + get_idx!(SA[$a - 1]), SA, PA + SA[$a], depth) == 0))
            {
                SA[$a] = !SA[$a];
            }
            if ($c & 4 > 0)
                && (ss_compare(T, SA, PA + get_idx!(SA[$b - 1]), SA, PA + SA[$b], depth) == 0)
            {
                SA[$b] = !SA[$b];
            }
        };
    }

    let mut stack = MergeStack::new();
    let mut l: SAPtr;
    let mut r: SAPtr;
    let mut lm: SAPtr;
    let mut rm: SAPtr;
    let mut m: Idx;
    let mut len: Idx;
    let mut half: Idx;
    let mut check: Idx;
    let mut next: Idx;

    // BARBARIAN
    check = 0;
    loop {
        if (last - middle) <= bufsize {
            if (first < middle) && (middle < last) {
                ss_mergebackward(T, SA, PA, first, middle, last, buf, depth);
            }
            merge_check!(first, last, check);
            if !stack
                .pop(&mut first, &mut middle, &mut last, &mut check)
                .is_ok()
            {
                return;
            }
            continue;
        }

        if (middle - first) <= bufsize {
            if first < middle {
                ss_mergeforward(T, SA, PA, first, middle, last, buf, depth);
            }
            merge_check!(first, last, check);
            if !stack
                .pop(&mut first, &mut middle, &mut last, &mut check)
                .is_ok()
            {
                return;
            }
            continue;
        }

        // OLANNA
        m = 0;
        len = cmp::min((middle - first).0, (last - middle).0);
        half = len >> 1;
        while 0 < len {
            if ss_compare(
                T,
                SA,
                PA + get_idx!(SA[middle + m + half]),
                SA,
                PA + get_idx!(SA[middle - m - half - 1]),
                depth,
            ) < 0
            {
                m += half + 1;
                half -= (len & 1) ^ 1;
            }

            // iter
            half >>= 1;
            len = half;
        }

        if 0 < m {
            lm = middle - m;
            rm = middle + m;
            ss_blockswap(SA, lm, middle, m);
            r = middle;
            l = middle;
            next = 0;
            if rm < last {
                if SA[rm] < 0 {
                    SA[rm] = !SA[rm];
                    if first < lm {
                        // KOOPA
                        l -= 1;
                        while SA[l] < 0 {
                            l -= 1;
                        }
                        next |= 4;
                    }
                } else if first < lm {
                    // MUNCHER
                    while SA[r] < 0 {
                        r += 1;
                    }
                    next |= 2;
                }
            }

            if (l - first) <= (last - r) {
                stack.push(r, rm, last, (next & 3) | (check & 4));
                middle = lm;
                last = l;
                check = (check & 3) | (next & 4);
            } else {
                if (next & 2 > 0) && (r == middle) {
                    next ^= 6
                }
                stack.push(first, lm, l, (check & 3) | (next & 4));
                first = r;
                middle = rm;
                check = (next & 3) | (check & 4);
            }
        } else {
            if ss_compare(
                T,
                SA,
                PA + get_idx!(SA[middle - 1]),
                SA,
                PA + SA[middle],
                depth,
            ) == 0
            {
                SA[middle] = !SA[middle];
            }
            merge_check!(first, last, check);
            if !stack
                .pop(&mut first, &mut middle, &mut last, &mut check)
                .is_ok()
            {
                return;
            }
        }
    }
}

//------------------------------------------------------------------------------

const STACK_SIZE: usize = 16;

struct StackItem {
    a: SAPtr,
    b: SAPtr,
    c: Idx,
    d: Idx,
}

impl Default for StackItem {
    fn default() -> Self {
        Self {
            a: SAPtr(0),
            b: SAPtr(0),
            c: 0,
            d: 0,
        }
    }
}

struct Stack {
    items: [StackItem; STACK_SIZE],
    size: usize,
}

impl Stack {
    fn new() -> Self {
        Self {
            items: Default::default(),
            size: 0,
        }
    }

    #[inline(always)]
    fn push(&mut self, a: SAPtr, b: SAPtr, c: Idx, d: Idx) {
        assert!(self.size < STACK_SIZE);
        self.items[self.size].a = a;
        self.items[self.size].b = b;
        self.items[self.size].c = c;
        self.items[self.size].d = d;
        self.size += 1;
    }

    #[inline(always)]
    #[must_use]
    fn pop(&mut self, a: &mut SAPtr, b: &mut SAPtr, c: &mut Idx, d: &mut Idx) -> Result<(), ()> {
        if (self.size == 0) {
            Err(())
        } else {
            self.size -= 1;
            *a = self.items[self.size].a;
            *b = self.items[self.size].b;
            *c = self.items[self.size].c;
            *d = self.items[self.size].d;
            Ok(())
        }
    }
}

//------------------------------------------------------------------------------

//--------------------
// Function
//--------------------

/// Substring sort
pub fn sssort(
    T: &Text,
    SA: &mut SuffixArray,
    PA: SAPtr,
    mut first: SAPtr,
    last: SAPtr,
    mut buf: SAPtr,
    mut bufsize: Idx,
    depth: Idx,
    n: Idx,
    lastsuffix: bool,
) {
    // Note: in most of this file "PA" seems to mean "Partition Array" - we're
    // working on a slice of SA. This is also why SA (or a mutable reference to it)
    // is passed around, so we don't run into lifetime issues.

    let mut a: SAPtr;
    let mut b: SAPtr;
    let mut middle: SAPtr;
    let mut curbuf: SAPtr;
    let mut j: Idx;
    let mut k: Idx;
    let mut curbufsize: Idx;
    let mut limit: Idx;
    let mut i: Idx;

    if lastsuffix {
        first += 1;
    }

    // 🎃

    limit = ss_isqrt(last - first);
    if ((bufsize < SS_BLOCKSIZE) && (bufsize < (last - first)) && (bufsize < limit)) {
        if (SS_BLOCKSIZE < limit) {
            limit = SS_BLOCKSIZE;
        }
        middle = last - limit;
        buf = middle;
        bufsize = limit;
    } else {
        middle = last;
        limit = 0;
    }

    // ☕

    a = first;
    i = 0;
    while SS_BLOCKSIZE < (middle - a) {
        ss_mintrosort(T, SA, PA, a, a + SS_BLOCKSIZE, depth);

        curbufsize = (last - (a + SS_BLOCKSIZE)).into();
        curbuf = a + SS_BLOCKSIZE;
        if (curbufsize <= bufsize) {
            curbufsize = bufsize;
            curbuf = buf;
        }

        b = a;
        k = SS_BLOCKSIZE;
        j = i;
        while (j & 1) > 0 {
            ss_swapmerge(T, SA, PA, b - k, b, b + k, curbuf, curbufsize, depth);

            // iter
            b -= k;
            k <<= 1;
            j >>= 1;
        }

        // iter
        a += SS_BLOCKSIZE;
        i += 1;
    }

    ss_mintrosort(T, SA, PA, a, middle, depth);

    // 😈

    k = SS_BLOCKSIZE;
    while i != 0 {
        if (i & 1) > 0 {
            ss_swapmerge(T, SA, PA, a - k, a, middle, buf, bufsize, depth);
            a -= k;
        }

        // iter
        k <<= 1;
        i >>= 1;
    }

    if limit != 0 {
        ss_mintrosort(T, SA, PA, middle, last, depth);
        ss_inplacemerge(T, SA, PA, first, middle, last, depth);
    }

    if lastsuffix {
        // Insert last type B* suffix
        let mut PAi: [Idx; 2] = [SA[PA + SA[first - 1]], n - 2];
        let SAI = SuffixArray(&mut PAi);

        a = first;
        i = SA[first - 1];

        // CELINE
        while (a < last)
            && ((SA[a] < 0) || (0 < ss_compare(T, &SAI, SAPtr(0), SA, PA + SA[a], depth)))
        {
            // body
            SA[a - 1] = SA[a];

            // iter
            a += 1;
        }
        SA[a - 1] = i;
    }
}
