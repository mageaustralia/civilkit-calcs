//! Eigenvalues and eigenvectors of a general (not symmetric) real square matrix.
//!
//! The symmetric solver in `matrix_solvers.rs` stays the path for symmetric input. A full matrix that is not
//! symmetric is reduced to upper Hessenberg form by Householder similarity transformations and then to real Schur
//! form by the shifted double QR (Francis) iteration; the eigenvectors are found by back substitution on the Schur
//! form and transformed back (the classic orthes / hqr2 pair of the public domain EISPACK and JAMA routines).
//! Plain `f64` throughout: the caller converts every element to one unit first.

/// One eigenpair: the eigenvalue `(re, im)` and its eigenvector, one `(re, im)` per component.
pub type EigenPair = ((f64, f64), Vec<(f64, f64)>);

/// All eigenpairs of the square matrix `a` (rows), sorted by real part and then imaginary part, both ascending.
/// Each eigenvector has unit length and is turned so that its largest component is real and positive.
/// `None` if the iteration does not converge (it always does for finite input in practice).
pub fn eigen_pairs(a: &[Vec<f64>]) -> Option<Vec<EigenPair>> {
    let n = a.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let mut h: Vec<Vec<f64>> = a.to_vec();
    let mut v = vec![vec![0.0; n]; n];
    let mut d = vec![0.0; n];
    let mut e = vec![0.0; n];
    orthes(&mut h, &mut v);
    if !hqr2(&mut h, &mut v, &mut d, &mut e) {
        return None;
    }
    let mut pairs: Vec<EigenPair> = Vec::with_capacity(n);
    let mut j = 0;
    while j < n {
        if e[j] == 0.0 {
            pairs.push(((d[j], 0.0), (0..n).map(|i| (v[i][j], 0.0)).collect()));
            j += 1;
        } else {
            // Columns j and j + 1 hold the real and imaginary parts of the vector of d[j] + i e[j]; the next
            // eigenvalue is its conjugate.
            let x: Vec<(f64, f64)> = (0..n).map(|i| (v[i][j], v[i][j + 1])).collect();
            pairs.push(((d[j], e[j]), x.clone()));
            pairs.push(((d[j + 1], e[j + 1]), x.iter().map(|c| (c.0, -c.1)).collect()));
            j += 2;
        }
    }
    for p in pairs.iter_mut() {
        normalize(&mut p.1);
    }
    pairs.sort_by(|p, q| {
        let o = p.0 .0.partial_cmp(&q.0 .0).unwrap_or(std::cmp::Ordering::Equal);
        o.then(p.0 .1.partial_cmp(&q.0 .1).unwrap_or(std::cmp::Ordering::Equal))
    });
    Some(pairs)
}

/// Unit length, and the largest component rotated onto the positive real axis (so the result does not depend on
/// the arbitrary scale the back substitution leaves).
fn normalize(x: &mut [(f64, f64)]) {
    let mut big = (0.0, 0.0);
    let mut big_abs = -1.0;
    let mut norm = 0.0;
    for c in x.iter() {
        let m = c.0 * c.0 + c.1 * c.1;
        norm += m;
        if m > big_abs * (1.0 + 1e-12) {
            big_abs = m;
            big = *c;
        }
    }
    if norm == 0.0 {
        return;
    }
    let r = big_abs.sqrt();
    // Multiply by conj(big) / (|big| |x|).
    let (cr, ci) = (big.0 / r, -big.1 / r);
    let s = 1.0 / norm.sqrt();
    for c in x.iter_mut() {
        let re = (c.0 * cr - c.1 * ci) * s;
        let im = (c.0 * ci + c.1 * cr) * s;
        *c = (
            if re == 0.0 { 0.0 } else { re },
            if im.abs() < 1e-15 { 0.0 } else { im },
        );
    }
}

/// Householder reduction to upper Hessenberg form; `v` receives the accumulated transformation.
fn orthes(h: &mut [Vec<f64>], v: &mut [Vec<f64>]) {
    let n = h.len();
    let high = n - 1;
    let mut ort = vec![0.0; n];
    for m in 1..high {
        let scale: f64 = (m..=high).map(|i| h[i][m - 1].abs()).sum();
        if scale == 0.0 {
            continue;
        }
        let mut hh = 0.0;
        for i in (m..=high).rev() {
            ort[i] = h[i][m - 1] / scale;
            hh += ort[i] * ort[i];
        }
        let mut g = hh.sqrt();
        if ort[m] > 0.0 {
            g = -g;
        }
        hh -= ort[m] * g;
        ort[m] -= g;
        for j in m..n {
            let f = (m..=high).rev().map(|i| ort[i] * h[i][j]).sum::<f64>() / hh;
            for i in m..=high {
                h[i][j] -= f * ort[i];
            }
        }
        for row in h.iter_mut().take(high + 1) {
            let f = (m..=high).rev().map(|j| ort[j] * row[j]).sum::<f64>() / hh;
            for j in m..=high {
                row[j] -= f * ort[j];
            }
        }
        ort[m] *= scale;
        h[m][m - 1] = scale * g;
    }
    for (i, row) in v.iter_mut().enumerate() {
        for (j, x) in row.iter_mut().enumerate() {
            *x = if i == j { 1.0 } else { 0.0 };
        }
    }
    for m in (1..high).rev() {
        if h[m][m - 1] == 0.0 {
            continue;
        }
        for i in m + 1..=high {
            ort[i] = h[i][m - 1];
        }
        for j in m..=high {
            let mut g: f64 = (m..=high).map(|i| ort[i] * v[i][j]).sum();
            g = (g / ort[m]) / h[m][m - 1];
            for i in m..=high {
                v[i][j] += g * ort[i];
            }
        }
    }
}

/// Complex division `(xr + i xi) / (yr + i yi)`.
fn cdiv(xr: f64, xi: f64, yr: f64, yi: f64) -> (f64, f64) {
    if yr.abs() > yi.abs() {
        let r = yi / yr;
        let d = yr + r * yi;
        ((xr + r * xi) / d, (xi - r * xr) / d)
    } else {
        let r = yr / yi;
        let d = yi + r * yr;
        ((r * xr + xi) / d, (r * xi - xr) / d)
    }
}

/// Shifted double QR iteration from Hessenberg to real Schur form, then the eigenvectors by back substitution.
/// Eigenvalues land in `d` (real parts) and `e` (imaginary parts); `v` becomes the eigenvector matrix (for a
/// complex pair, two columns: real and imaginary part). Returns false if the iteration does not converge.
#[allow(clippy::many_single_char_names)]
fn hqr2(hm: &mut [Vec<f64>], vm: &mut [Vec<f64>], d: &mut [f64], e: &mut [f64]) -> bool {
    let nn = hm.len() as isize;
    let eps = f64::EPSILON;
    let (low, high) = (0isize, nn - 1);
    macro_rules! h {
        ($i:expr, $j:expr) => {
            hm[($i) as usize][($j) as usize]
        };
    }
    macro_rules! v {
        ($i:expr, $j:expr) => {
            vm[($i) as usize][($j) as usize]
        };
    }
    let mut n = nn - 1;
    let mut exshift = 0.0;
    let (mut p, mut q, mut r, mut s, mut z) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let (mut t, mut w, mut x, mut y): (f64, f64, f64, f64);
    let mut norm = 0.0;
    for i in 0..nn {
        for j in (i - 1).max(0)..nn {
            norm += h!(i, j).abs();
        }
    }
    let mut iter = 0;
    let mut total = 0;
    while n >= low {
        let mut l = n;
        while l > low {
            s = h!(l - 1, l - 1).abs() + h!(l, l).abs();
            if s == 0.0 {
                s = norm;
            }
            if h!(l, l - 1).abs() < eps * s {
                break;
            }
            l -= 1;
        }
        if l == n {
            // One root found.
            h!(n, n) += exshift;
            d[n as usize] = h!(n, n);
            e[n as usize] = 0.0;
            n -= 1;
            iter = 0;
        } else if l == n - 1 {
            // Two roots found.
            w = h!(n, n - 1) * h!(n - 1, n);
            p = (h!(n - 1, n - 1) - h!(n, n)) / 2.0;
            q = p * p + w;
            z = q.abs().sqrt();
            h!(n, n) += exshift;
            h!(n - 1, n - 1) += exshift;
            x = h!(n, n);
            if q >= 0.0 {
                // A real pair.
                z = if p >= 0.0 { p + z } else { p - z };
                d[(n - 1) as usize] = x + z;
                d[n as usize] = d[(n - 1) as usize];
                if z != 0.0 {
                    d[n as usize] = x - w / z;
                }
                e[(n - 1) as usize] = 0.0;
                e[n as usize] = 0.0;
                x = h!(n, n - 1);
                s = x.abs() + z.abs();
                p = x / s;
                q = z / s;
                r = (p * p + q * q).sqrt();
                p /= r;
                q /= r;
                for j in n - 1..nn {
                    z = h!(n - 1, j);
                    h!(n - 1, j) = q * z + p * h!(n, j);
                    h!(n, j) = q * h!(n, j) - p * z;
                }
                for i in 0..=n {
                    z = h!(i, n - 1);
                    h!(i, n - 1) = q * z + p * h!(i, n);
                    h!(i, n) = q * h!(i, n) - p * z;
                }
                for i in low..=high {
                    z = v!(i, n - 1);
                    v!(i, n - 1) = q * z + p * v!(i, n);
                    v!(i, n) = q * v!(i, n) - p * z;
                }
            } else {
                // A complex pair.
                d[(n - 1) as usize] = x + p;
                d[n as usize] = x + p;
                e[(n - 1) as usize] = z;
                e[n as usize] = -z;
            }
            n -= 2;
            iter = 0;
        } else {
            // No convergence yet: one more double shift step.
            total += 1;
            if total > 100 * nn.max(10) {
                return false;
            }
            x = h!(n, n);
            y = 0.0;
            w = 0.0;
            if l < n {
                y = h!(n - 1, n - 1);
                w = h!(n, n - 1) * h!(n - 1, n);
            }
            // Exceptional shifts after 10 and 30 steps without a split.
            if iter == 10 {
                exshift += x;
                for i in low..=n {
                    h!(i, i) -= x;
                }
                s = h!(n, n - 1).abs() + h!(n - 1, n - 2).abs();
                x = 0.75 * s;
                y = x;
                w = -0.4375 * s * s;
            }
            if iter == 30 {
                s = (y - x) / 2.0;
                s = s * s + w;
                if s > 0.0 {
                    s = s.sqrt();
                    if y < x {
                        s = -s;
                    }
                    s = x - w / ((y - x) / 2.0 + s);
                    for i in low..=n {
                        h!(i, i) -= s;
                    }
                    exshift += s;
                    x = 0.964;
                    y = x;
                    w = x;
                }
            }
            iter += 1;
            // Look for two consecutive small subdiagonal elements.
            let mut m = n - 2;
            while m >= l {
                z = h!(m, m);
                r = x - z;
                s = y - z;
                p = (r * s - w) / h!(m + 1, m) + h!(m, m + 1);
                q = h!(m + 1, m + 1) - z - r - s;
                r = h!(m + 2, m + 1);
                s = p.abs() + q.abs() + r.abs();
                p /= s;
                q /= s;
                r /= s;
                if m == l {
                    break;
                }
                if h!(m, m - 1).abs() * (q.abs() + r.abs())
                    < eps * (p.abs() * (h!(m - 1, m - 1).abs() + z.abs() + h!(m + 1, m + 1).abs()))
                {
                    break;
                }
                m -= 1;
            }
            for i in m + 2..=n {
                h!(i, i - 2) = 0.0;
                if i > m + 2 {
                    h!(i, i - 3) = 0.0;
                }
            }
            // Double QR step on rows l..n and columns m..n.
            let mut k = m;
            while k < n {
                let notlast = k != n - 1;
                if k != m {
                    p = h!(k, k - 1);
                    q = h!(k + 1, k - 1);
                    r = if notlast { h!(k + 2, k - 1) } else { 0.0 };
                    x = p.abs() + q.abs() + r.abs();
                    if x == 0.0 {
                        k += 1;
                        continue;
                    }
                    p /= x;
                    q /= x;
                    r /= x;
                }
                s = (p * p + q * q + r * r).sqrt();
                if p < 0.0 {
                    s = -s;
                }
                if s != 0.0 {
                    if k != m {
                        h!(k, k - 1) = -s * x;
                    } else if l != m {
                        h!(k, k - 1) = -h!(k, k - 1);
                    }
                    p += s;
                    x = p / s;
                    y = q / s;
                    z = r / s;
                    q /= p;
                    r /= p;
                    for j in k..nn {
                        p = h!(k, j) + q * h!(k + 1, j);
                        if notlast {
                            p += r * h!(k + 2, j);
                            h!(k + 2, j) -= p * z;
                        }
                        h!(k, j) -= p * x;
                        h!(k + 1, j) -= p * y;
                    }
                    for i in 0..=n.min(k + 3) {
                        p = x * h!(i, k) + y * h!(i, k + 1);
                        if notlast {
                            p += z * h!(i, k + 2);
                            h!(i, k + 2) -= p * r;
                        }
                        h!(i, k) -= p;
                        h!(i, k + 1) -= p * q;
                    }
                    for i in low..=high {
                        p = x * v!(i, k) + y * v!(i, k + 1);
                        if notlast {
                            p += z * v!(i, k + 2);
                            v!(i, k + 2) -= p * r;
                        }
                        v!(i, k) -= p;
                        v!(i, k + 1) -= p * q;
                    }
                }
                k += 1;
            }
        }
    }
    if norm == 0.0 {
        return true;
    }
    // Back substitution on the Schur form.
    n = nn - 1;
    while n >= 0 {
        p = d[n as usize];
        q = e[n as usize];
        if q == 0.0 {
            // A real vector.
            let mut l = n;
            h!(n, n) = 1.0;
            let mut i = n - 1;
            while i >= 0 {
                w = h!(i, i) - p;
                r = 0.0;
                for j in l..=n {
                    r += h!(i, j) * h!(j, n);
                }
                if e[i as usize] < 0.0 {
                    z = w;
                    s = r;
                } else {
                    l = i;
                    if e[i as usize] == 0.0 {
                        h!(i, n) = if w != 0.0 { -r / w } else { -r / (eps * norm) };
                    } else {
                        x = h!(i, i + 1);
                        y = h!(i + 1, i);
                        q = (d[i as usize] - p) * (d[i as usize] - p) + e[i as usize] * e[i as usize];
                        t = (x * s - z * r) / q;
                        h!(i, n) = t;
                        h!(i + 1, n) = if x.abs() > z.abs() {
                            (-r - w * t) / x
                        } else {
                            (-s - y * t) / z
                        };
                    }
                    t = h!(i, n).abs();
                    if (eps * t) * t > 1.0 {
                        for j in i..=n {
                            h!(j, n) /= t;
                        }
                    }
                }
                i -= 1;
            }
        } else if q < 0.0 {
            // A complex vector: real part in column n - 1, imaginary part in column n.
            let mut l = n - 1;
            if h!(n, n - 1).abs() > h!(n - 1, n).abs() {
                h!(n - 1, n - 1) = q / h!(n, n - 1);
                h!(n - 1, n) = -(h!(n, n) - p) / h!(n, n - 1);
            } else {
                let c = cdiv(0.0, -h!(n - 1, n), h!(n - 1, n - 1) - p, q);
                h!(n - 1, n - 1) = c.0;
                h!(n - 1, n) = c.1;
            }
            h!(n, n - 1) = 0.0;
            h!(n, n) = 1.0;
            let mut i = n - 2;
            while i >= 0 {
                let (mut ra, mut sa) = (0.0, 0.0);
                for j in l..=n {
                    ra += h!(i, j) * h!(j, n - 1);
                    sa += h!(i, j) * h!(j, n);
                }
                w = h!(i, i) - p;
                if e[i as usize] < 0.0 {
                    z = w;
                    r = ra;
                    s = sa;
                } else {
                    l = i;
                    if e[i as usize] == 0.0 {
                        let c = cdiv(-ra, -sa, w, q);
                        h!(i, n - 1) = c.0;
                        h!(i, n) = c.1;
                    } else {
                        x = h!(i, i + 1);
                        y = h!(i + 1, i);
                        let di = d[i as usize] - p;
                        let mut vr = di * di + e[i as usize] * e[i as usize] - q * q;
                        let vi = di * 2.0 * q;
                        if vr == 0.0 && vi == 0.0 {
                            vr = eps * norm * (w.abs() + q.abs() + x.abs() + y.abs() + z.abs());
                        }
                        let c = cdiv(x * r - z * ra + q * sa, x * s - z * sa - q * ra, vr, vi);
                        h!(i, n - 1) = c.0;
                        h!(i, n) = c.1;
                        if x.abs() > z.abs() + q.abs() {
                            h!(i + 1, n - 1) = (-ra - w * h!(i, n - 1) + q * h!(i, n)) / x;
                            h!(i + 1, n) = (-sa - w * h!(i, n) - q * h!(i, n - 1)) / x;
                        } else {
                            let c = cdiv(-r - y * h!(i, n - 1), -s - y * h!(i, n), z, q);
                            h!(i + 1, n - 1) = c.0;
                            h!(i + 1, n) = c.1;
                        }
                    }
                    t = h!(i, n - 1).abs().max(h!(i, n).abs());
                    if (eps * t) * t > 1.0 {
                        for j in i..=n {
                            h!(j, n - 1) /= t;
                            h!(j, n) /= t;
                        }
                    }
                }
                i -= 1;
            }
        }
        n -= 1;
    }
    // Back to the eigenvectors of the original matrix.
    let mut j = nn - 1;
    while j >= low {
        for i in low..=high {
            z = 0.0;
            for k in low..=j.min(high) {
                z += v!(i, k) * h!(k, j);
            }
            v!(i, j) = z;
        }
        j -= 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// max |A x - lambda x| over all pairs.
    fn residual(a: &[Vec<f64>], pairs: &[EigenPair]) -> f64 {
        let n = a.len();
        let mut worst = 0.0f64;
        for ((lr, li), x) in pairs {
            for i in 0..n {
                let (mut sr, mut si) = (0.0, 0.0);
                for j in 0..n {
                    sr += a[i][j] * x[j].0;
                    si += a[i][j] * x[j].1;
                }
                let (tr, ti) = (lr * x[i].0 - li * x[i].1, lr * x[i].1 + li * x[i].0);
                worst = worst.max((sr - tr).abs()).max((si - ti).abs());
            }
        }
        worst
    }

    #[test]
    fn real_nonsymmetric() {
        let a = vec![vec![4.0, 1.0], vec![2.0, 3.0]];
        let p = eigen_pairs(&a).unwrap();
        assert!((p[0].0 .0 - 2.0).abs() < 1e-12 && p[0].0 .1 == 0.0);
        assert!((p[1].0 .0 - 5.0).abs() < 1e-12 && p[1].0 .1 == 0.0);
        assert!(residual(&a, &p) < 1e-12);
        // Unit length, largest component positive.
        let x = &p[1].1;
        assert!(((x[0].0 * x[0].0 + x[1].0 * x[1].0) - 1.0).abs() < 1e-12);
        assert!(x[0].0 > 0.0);
    }

    #[test]
    fn rotation_has_conjugate_pair() {
        let a = vec![vec![0.0, -1.0], vec![1.0, 0.0]];
        let p = eigen_pairs(&a).unwrap();
        assert!(p[0].0 .0.abs() < 1e-14 && (p[0].0 .1 + 1.0).abs() < 1e-14);
        assert!(p[1].0 .0.abs() < 1e-14 && (p[1].0 .1 - 1.0).abs() < 1e-14);
        assert!(residual(&a, &p) < 1e-12);
    }

    #[test]
    fn larger_mixed_spectrum() {
        // Block diagonal in disguise: eigenvalues 1 +- 2i, 3, -2 after a similarity transformation.
        let a = vec![
            vec![1.0, -2.0, 0.5, 0.0],
            vec![2.0, 1.0, 0.0, 0.3],
            vec![0.0, 0.0, 3.0, 1.0],
            vec![0.0, 0.0, 0.0, -2.0],
        ];
        let p = eigen_pairs(&a).unwrap();
        let ev: Vec<(f64, f64)> = p.iter().map(|q| q.0).collect();
        let want = [(-2.0, 0.0), (1.0, -2.0), (1.0, 2.0), (3.0, 0.0)];
        for (g, w) in ev.iter().zip(want.iter()) {
            assert!((g.0 - w.0).abs() < 1e-10 && (g.1 - w.1).abs() < 1e-10, "{ev:?}");
        }
        assert!(residual(&a, &p) < 1e-10);
    }

    #[test]
    fn repeated_and_defective_eigenvalues_terminate() {
        // A Jordan block has one eigenvector for a double eigenvalue; a zero matrix has every vector.
        for a in [
            vec![vec![1.0, 1.0], vec![0.0, 1.0]],
            vec![vec![0.0, 0.0], vec![0.0, 0.0]],
            vec![vec![2.0, 0.0], vec![0.0, 2.0]],
        ] {
            let p = eigen_pairs(&a).unwrap();
            assert_eq!(p.len(), 2);
            assert!(residual(&a, &p) < 1e-12, "{a:?}");
        }
        // Companion matrix of (x - 1)(x - 2)(x - 3): eigenvalues are the roots.
        let a = vec![vec![6.0, -11.0, 6.0], vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
        let p = eigen_pairs(&a).unwrap();
        for (k, q) in p.iter().enumerate() {
            assert!((q.0 .0 - (k + 1) as f64).abs() < 1e-10 && q.0 .1 == 0.0);
        }
        assert!(residual(&a, &p) < 1e-10);
    }

    #[test]
    fn random_matrices_satisfy_definition() {
        let mut seed = 12345u64;
        let mut rnd = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
        };
        for n in 1..9 {
            let a: Vec<Vec<f64>> = (0..n).map(|_| (0..n).map(|_| rnd()).collect()).collect();
            let p = eigen_pairs(&a).unwrap();
            assert_eq!(p.len(), n);
            assert!(residual(&a, &p) < 1e-10, "n = {n}");
            let trace: f64 = (0..n).map(|i| a[i][i]).sum();
            let sum: f64 = p.iter().map(|q| q.0 .0).sum();
            assert!((trace - sum).abs() < 1e-10);
        }
    }
}
