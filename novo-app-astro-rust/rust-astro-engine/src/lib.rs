//! Motor de Cálculo Astronômico e Astrológico em Rust (Portado de AA.js / Jean Meeus / VSOP87)
//! Inclui correções de bugs encontrados no AA.js original:
//! - Ordem dos argumentos de `fk5::get_correction_in_longitude` no cálculo do Sol (`sun.ts`)
//! - Soma do termo `B5 * rho^5` na latitude eclíptica de Júpiter (`jupiter/coordinates.ts`)

pub mod coeffs;

pub const J1970: f64 = 2440588.0;
pub const J2000: f64 = 2451545.0;
pub const DAYMS: f64 = 86_400_000.0;
pub const DEG2RAD: f64 = std::f64::consts::PI / 180.0;
pub const RAD2DEG: f64 = 180.0 / std::f64::consts::PI;
pub const RAD2H: f64 = 12.0 / std::f64::consts::PI;
pub const H2RAD: f64 = std::f64::consts::PI / 12.0;
pub const DEG2H: f64 = 1.0 / 15.0;

#[derive(Debug, Clone, Copy)]
pub struct EclipticCoordinates {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct EquatorialCoordinates {
    pub right_ascension: f64,
    pub declination: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct EllipticalDetails {
    pub apparent_light_time: f64,
    pub apparent_geocentric_distance: f64,
    pub apparent_geocentric_ecliptic: EclipticCoordinates,
    pub apparent_geocentric_equatorial: EquatorialCoordinates,
}

#[derive(Debug, Clone, Copy)]
pub struct Sexagesimal {
    pub radix: i32,
    pub minutes: i32,
    pub seconds: f64,
}

pub fn fmod_aa(a: f64, b: f64) -> f64 {
    a - (a / b).floor() * b
}

pub fn map_to_0_360(degrees: f64) -> f64 {
    let mut res = degrees.rem_euclid(360.0);
    if res < 0.0 {
        res += 360.0;
    }
    res
}

pub fn map_to_minus90_90(degrees: f64) -> f64 {
    let mut res = map_to_0_360(degrees);
    if res > 270.0 {
        res -= 360.0;
    } else if res > 90.0 {
        res = 180.0 - res;
    }
    res
}

pub fn get_decimal(d: f64, m: f64, s: f64, positive: bool) -> f64 {
    let val = d.abs() + m.abs() / 60.0 + s.abs() / 3600.0;
    if positive { val } else { -val }
}

pub fn get_sexagesimal(decimal: f64) -> Sexagesimal {
    let abs_val = decimal.abs();
    let degrees = abs_val.floor();
    let frac_deg = abs_val - degrees;
    let frac_min = frac_deg * 60.0;
    let minutes = frac_min.floor();
    let seconds = (frac_min - minutes) * 60.0;
    Sexagesimal {
        radix: degrees as i32,
        minutes: minutes as i32,
        seconds,
    }
}

pub mod julianday {
    use super::*;

    pub fn from_unix_ms(unix_ms: f64) -> f64 {
        unix_ms / DAYMS - 0.5 + J1970
    }

    pub fn to_unix_ms(jd: f64) -> f64 {
        (jd + 0.5 - J1970) * DAYMS
    }

    /// Calcula o Dia Juliano a partir de uma data UTC (ano, mes 1..12, dia, hora, min, seg)
    /// Fórmula clássica de Jean Meeus (Cap. 7)
    pub fn from_utc(year: i32, month: i32, day: i32, hour: i32, minute: i32, second: f64) -> f64 {
        let mut y = year as f64;
        let mut m = month as f64;
        if m <= 2.0 {
            y -= 1.0;
            m += 12.0;
        }
        let a = (y / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        let day_frac = (day as f64) + (hour as f64) / 24.0 + (minute as f64) / 1440.0 + second / 86400.0;
        (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day_frac + b - 1524.5
    }

    pub fn local_sidereal_time(jd: f64, lng_deg: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let gmst = 280.46061837 + 360.98564736629 * (jd - J2000) + 0.000387933 * t * t - t * t * t / 38710000.0;
        let gmst_pos = map_to_0_360(gmst);
        ((gmst_pos + lng_deg) * DEG2H).rem_euclid(24.0)
    }
}

pub mod nutation {
    use super::*;

    const NUTATION_COEFFS: &[(f64, f64, f64, f64, f64, f64, f64, f64, f64)] = &[
        (0.0, 0.0, 0.0, 0.0, 1.0, -171996.0, -174.2, 92025.0, 8.9),
        (-2.0, 0.0, 0.0, 2.0, 2.0, -13187.0, -1.6, 5736.0, -3.1),
        (0.0, 0.0, 0.0, 2.0, 2.0, -2274.0, -0.2, 977.0, -0.5),
        (0.0, 0.0, 0.0, 0.0, 2.0, 2062.0, 0.2, -895.0, 0.5),
        (0.0, 1.0, 0.0, 0.0, 0.0, 1426.0, -3.4, 54.0, -0.1),
        (0.0, 0.0, 1.0, 0.0, 0.0, 712.0, 0.1, -7.0, 0.0),
        (-2.0, 1.0, 0.0, 2.0, 2.0, -517.0, 1.2, 224.0, -0.6),
        (0.0, 0.0, 0.0, 2.0, 1.0, -386.0, -0.4, 200.0, 0.0),
        (0.0, 0.0, 1.0, 2.0, 2.0, -301.0, 0.0, 129.0, -0.1),
        (-2.0, -1.0, 0.0, 2.0, 2.0, 217.0, -0.5, -95.0, 0.3),
        (-2.0, 0.0, 1.0, 0.0, 0.0, -158.0, 0.0, 0.0, 0.0),
        (-2.0, 0.0, 0.0, 2.0, 1.0, 129.0, 0.1, -70.0, 0.0),
        (0.0, 0.0, -1.0, 2.0, 2.0, 123.0, 0.0, -53.0, 0.0),
        (2.0, 0.0, 0.0, 0.0, 0.0, 63.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 1.0, 0.0, 1.0, 63.0, 0.1, -33.0, 0.0),
        (2.0, 0.0, -1.0, 2.0, 2.0, -59.0, 0.0, 26.0, 0.0),
        (0.0, 0.0, -1.0, 0.0, 1.0, -58.0, -0.1, 32.0, 0.0),
        (0.0, 0.0, 1.0, 2.0, 1.0, -51.0, 0.0, 27.0, 0.0),
        (-2.0, 0.0, 2.0, 0.0, 0.0, 48.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, -2.0, 2.0, 1.0, 46.0, 0.0, -24.0, 0.0),
        (2.0, 0.0, 0.0, 2.0, 2.0, -38.0, 0.0, 16.0, 0.0),
        (0.0, 0.0, 2.0, 2.0, 2.0, -31.0, 0.0, 13.0, 0.0),
        (0.0, 0.0, 2.0, 0.0, 0.0, 29.0, 0.0, 0.0, 0.0),
        (-2.0, 0.0, 1.0, 2.0, 2.0, 29.0, 0.0, -12.0, 0.0),
        (0.0, 0.0, 0.0, 2.0, 0.0, 26.0, 0.0, 0.0, 0.0),
        (-2.0, 0.0, 0.0, 2.0, 0.0, -22.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, -1.0, 2.0, 1.0, 21.0, 0.0, -10.0, 0.0),
        (0.0, 2.0, 0.0, 0.0, 0.0, 17.0, -0.1, 0.0, 0.0),
        (2.0, 0.0, -1.0, 0.0, 1.0, 16.0, 0.0, -8.0, 0.0),
        (-2.0, 2.0, 0.0, 2.0, 2.0, -16.0, 0.1, 7.0, 0.0),
        (0.0, 1.0, 0.0, 0.0, 1.0, -15.0, 0.0, 9.0, 0.0),
        (-2.0, 0.0, 1.0, 0.0, 1.0, -13.0, 0.0, 7.0, 0.0),
        (0.0, -1.0, 0.0, 0.0, 1.0, -12.0, 0.0, 6.0, 0.0),
        (0.0, 0.0, 2.0, -2.0, 0.0, 11.0, 0.0, 0.0, 0.0),
        (2.0, 0.0, -1.0, 2.0, 1.0, -10.0, 0.0, 5.0, 0.0),
        (2.0, 0.0, 1.0, 2.0, 2.0, -8.0, 0.0, 3.0, 0.0),
        (0.0, 1.0, 0.0, 2.0, 2.0, 7.0, 0.0, -3.0, 0.0),
        (-2.0, 1.0, 1.0, 0.0, 0.0, -7.0, 0.0, 0.0, 0.0),
        (0.0, -1.0, 0.0, 2.0, 2.0, -7.0, 0.0, 3.0, 0.0),
        (2.0, 0.0, 0.0, 2.0, 1.0, -7.0, 0.0, 3.0, 0.0),
        (2.0, 0.0, 1.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0),
        (-2.0, 0.0, 2.0, 2.0, 2.0, 6.0, 0.0, -3.0, 0.0),
        (-2.0, 0.0, 1.0, 2.0, 1.0, 6.0, 0.0, -3.0, 0.0),
        (2.0, 0.0, -2.0, 0.0, 1.0, -6.0, 0.0, 3.0, 0.0),
        (2.0, 0.0, 0.0, 0.0, 1.0, -6.0, 0.0, 3.0, 0.0),
        (0.0, -1.0, 1.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0),
        (-2.0, -1.0, 0.0, 2.0, 1.0, -5.0, 0.0, 3.0, 0.0),
        (-2.0, 0.0, 0.0, 0.0, 1.0, -5.0, 0.0, 3.0, 0.0),
        (0.0, 0.0, 2.0, 2.0, 1.0, -5.0, 0.0, 3.0, 0.0),
        (-2.0, 0.0, 2.0, 0.0, 1.0, 4.0, 0.0, 0.0, 0.0),
        (-2.0, 1.0, 0.0, 2.0, 1.0, 4.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 1.0, -2.0, 0.0, 4.0, 0.0, 0.0, 0.0),
        (-1.0, 0.0, 1.0, 0.0, 0.0, -4.0, 0.0, 0.0, 0.0),
        (-2.0, 1.0, 0.0, 0.0, 0.0, -4.0, 0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0, 0.0, 0.0, -4.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, -2.0, 2.0, 2.0, -3.0, 0.0, 0.0, 0.0),
        (-1.0, -1.0, 1.0, 0.0, 0.0, -3.0, 0.0, 0.0, 0.0),
        (0.0, 1.0, 1.0, 0.0, 0.0, -3.0, 0.0, 0.0, 0.0),
        (0.0, -1.0, 1.0, 2.0, 2.0, -3.0, 0.0, 0.0, 0.0),
        (2.0, -1.0, -1.0, 2.0, 2.0, -3.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 3.0, 2.0, 2.0, -3.0, 0.0, 0.0, 0.0),
        (2.0, -1.0, 0.0, 2.0, 2.0, -3.0, 0.0, 0.0, 0.0),
    ];

    fn fundamental_args(jd: f64) -> (f64, f64, f64, f64, f64, f64) {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        let d = map_to_0_360(297.85036 + 445267.111480 * t - 0.0019142 * t2 + t3 / 189474.0);
        let m = map_to_0_360(357.52772 + 35999.050340 * t - 0.0001603 * t2 - t3 / 300000.0);
        let mprime = map_to_0_360(134.96298 + 477198.867398 * t + 0.0086972 * t2 + t3 / 56250.0);
        let f = map_to_0_360(93.27191 + 483202.017538 * t - 0.0036825 * t2 + t3 / 327270.0);
        let omega = map_to_0_360(125.04452 - 1934.136261 * t + 0.0020708 * t2 + t3 / 450000.0);
        (t, d, m, mprime, f, omega)
    }

    pub fn get_nutation_in_longitude(jd: f64) -> f64 {
        let (t, d, m, mprime, f, omega) = fundamental_args(jd);
        let mut value = 0.0;
        for c in NUTATION_COEFFS {
            let arg = c.0 * d + c.1 * m + c.2 * mprime + c.3 * f + c.4 * omega;
            value += (c.5 + c.6 * t) * (arg * DEG2RAD).sin() * 0.0001;
        }
        value
    }

    pub fn get_nutation_in_obliquity(jd: f64) -> f64 {
        let (t, d, m, mprime, f, omega) = fundamental_args(jd);
        let mut value = 0.0;
        for c in NUTATION_COEFFS {
            let arg = c.0 * d + c.1 * m + c.2 * mprime + c.3 * f + c.4 * omega;
            value += (c.7 + c.8 * t) * (arg * DEG2RAD).cos() * 0.0001;
        }
        value
    }

    pub fn get_mean_obliquity_of_ecliptic(jd: f64) -> f64 {
        let u = (jd - J2000) / 3652500.0;
        let u2 = u * u;
        let u3 = u2 * u;
        let u4 = u3 * u;
        let u5 = u4 * u;
        let u6 = u5 * u;
        let u7 = u6 * u;
        let u8 = u7 * u;
        let u9 = u8 * u;
        let u10 = u9 * u;

        get_decimal(23.0, 26.0, 21.448, true)
            - get_decimal(0.0, 0.0, 4680.93, true) * u
            - get_decimal(0.0, 0.0, 1.55, true) * u2
            + get_decimal(0.0, 0.0, 1999.25, true) * u3
            - get_decimal(0.0, 0.0, 51.38, true) * u4
            - get_decimal(0.0, 0.0, 249.67, true) * u5
            - get_decimal(0.0, 0.0, 39.05, true) * u6
            + get_decimal(0.0, 0.0, 7.12, true) * u7
            + get_decimal(0.0, 0.0, 27.87, true) * u8
            + get_decimal(0.0, 0.0, 5.79, true) * u9
            + get_decimal(0.0, 0.0, 2.45, true) * u10
    }

    pub fn get_true_obliquity_of_ecliptic(jd: f64) -> f64 {
        let nut_obliq = get_nutation_in_obliquity(jd);
        get_mean_obliquity_of_ecliptic(jd) + get_decimal(0.0, 0.0, nut_obliq.abs(), nut_obliq >= 0.0)
    }
}

pub mod fk5 {
    use super::*;

    pub fn get_correction_in_longitude(jd: f64, lng: f64, lat: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let ldash = lng - 1.397 * t - 0.00031 * t * t;
        let value = -0.09033 + 0.03916 * ((ldash * DEG2RAD).cos() + (ldash * DEG2RAD).sin()) * (lat * DEG2RAD).tan();
        get_decimal(0.0, 0.0, value.abs(), value >= 0.0)
    }

    pub fn get_correction_in_latitude(jd: f64, lng: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let ldash = lng - 1.397 * t - 0.00031 * t * t;
        let value = 0.03916 * ((ldash * DEG2RAD).cos() - (ldash * DEG2RAD).sin());
        get_decimal(0.0, 0.0, value.abs(), value >= 0.0)
    }
}

pub mod coordinates {
    use super::*;

    pub fn declination_from_ecliptic(l: f64, b: f64, epsilon: f64) -> f64 {
        ((b * DEG2RAD).sin() * (epsilon * DEG2RAD).cos()
            + (b * DEG2RAD).cos() * (epsilon * DEG2RAD).sin() * (l * DEG2RAD).sin())
        .asin()
            * RAD2DEG
    }

    pub fn right_ascension_from_ecliptic(l: f64, b: f64, epsilon: f64) -> f64 {
        let y = (l * DEG2RAD).sin() * (epsilon * DEG2RAD).cos() - (b * DEG2RAD).tan() * (epsilon * DEG2RAD).sin();
        let x = (l * DEG2RAD).cos();
        (y.atan2(x) * RAD2DEG * DEG2H).rem_euclid(24.0)
    }

    pub fn transform_ecliptic_to_equatorial(l: f64, b: f64, epsilon: f64) -> EquatorialCoordinates {
        EquatorialCoordinates {
            right_ascension: right_ascension_from_ecliptic(l, b, epsilon),
            declination: declination_from_ecliptic(l, b, epsilon),
        }
    }
}

#[inline]
pub fn sum_vsop(coeffs: &[(f64, f64, f64)], rho: f64) -> f64 {
    let mut sum = 0.0;
    for &(a, b, c) in coeffs {
        sum += a * (b + c * rho).cos();
    }
    sum
}

pub mod earth {
    use super::*;
    use crate::coeffs::earth::*;

    pub fn get_ecliptic_longitude(jd: f64) -> f64 {
        let rho = (jd - J2000) / 365250.0;
        let rho2 = rho * rho;
        let rho3 = rho2 * rho;
        let rho4 = rho3 * rho;
        let rho5 = rho4 * rho;

        let l0 = sum_vsop(gL0EarthCoefficients, rho);
        let l1 = sum_vsop(gL1EarthCoefficients, rho);
        let l2 = sum_vsop(gL2EarthCoefficients, rho);
        let l3 = sum_vsop(gL3EarthCoefficients, rho);
        let l4 = sum_vsop(gL4EarthCoefficients, rho);
        let l5 = sum_vsop(gL5EarthCoefficients, rho);

        let value = (l0 + l1 * rho + l2 * rho2 + l3 * rho3 + l4 * rho4 + l5 * rho5) / 100_000_000.0;
        map_to_0_360(value * RAD2DEG)
    }

    pub fn get_ecliptic_latitude(jd: f64) -> f64 {
        let rho = (jd - J2000) / 365250.0;
        let b0 = sum_vsop(gB0EarthCoefficients, rho);
        let b1 = sum_vsop(gB1EarthCoefficients, rho);
        let value = (b0 + b1 * rho) / 100_000_000.0;
        map_to_minus90_90(value * RAD2DEG)
    }

    pub fn get_radius_vector(jd: f64) -> f64 {
        let rho = (jd - J2000) / 365250.0;
        let rho2 = rho * rho;
        let rho3 = rho2 * rho;
        let rho4 = rho3 * rho;

        let r0 = sum_vsop(gR0EarthCoefficients, rho);
        let r1 = sum_vsop(gR1EarthCoefficients, rho);
        let r2 = sum_vsop(gR2EarthCoefficients, rho);
        let r3 = sum_vsop(gR3EarthCoefficients, rho);
        let r4 = sum_vsop(gR4EarthCoefficients, rho);

        (r0 + r1 * rho + r2 * rho2 + r3 * rho3 + r4 * rho4) / 100_000_000.0
    }

    pub fn get_sun_mean_anomaly(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        map_to_0_360(357.5291092 + 35999.0502909 * t - 0.0001536 * t2 + t3 / 24490000.0)
    }

    pub fn get_eccentricity(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        1.0 - 0.002516 * t - 0.0000074 * t2
    }
}

pub mod sun {
    use super::*;

    pub fn get_geometric_ecliptic_longitude(jd: f64) -> f64 {
        map_to_0_360(earth::get_ecliptic_longitude(jd) + 180.0)
    }

    pub fn get_geometric_ecliptic_latitude(jd: f64) -> f64 {
        -earth::get_ecliptic_latitude(jd)
    }

    /// Corrigido em relação ao AA.js original: passa (jd, longitude, latitude) na ordem correta para fk5!
    pub fn get_geometric_fk5_ecliptic_longitude(jd: f64) -> f64 {
        let mut longitude = get_geometric_ecliptic_longitude(jd);
        let latitude = get_geometric_ecliptic_latitude(jd);
        longitude += fk5::get_correction_in_longitude(jd, longitude, latitude);
        longitude
    }

    pub fn get_geometric_fk5_ecliptic_latitude(jd: f64) -> f64 {
        let longitude = get_geometric_ecliptic_longitude(jd);
        let mut latitude = get_geometric_ecliptic_latitude(jd);
        latitude += fk5::get_correction_in_latitude(jd, longitude);
        latitude
    }

    pub fn get_apparent_ecliptic_longitude(jd: f64) -> f64 {
        let mut longitude = get_geometric_fk5_ecliptic_longitude(jd);
        let nut_long = nutation::get_nutation_in_longitude(jd);
        longitude += get_decimal(0.0, 0.0, nut_long.abs(), nut_long >= 0.0);
        let r = earth::get_radius_vector(jd);
        longitude -= get_decimal(0.0, 0.0, 20.4898 / r, true);
        map_to_0_360(longitude)
    }

    pub fn get_apparent_ecliptic_latitude(jd: f64) -> f64 {
        get_geometric_fk5_ecliptic_latitude(jd)
    }

    pub fn get_apparent_ecliptic_coordinates(jd: f64) -> EclipticCoordinates {
        EclipticCoordinates {
            longitude: get_apparent_ecliptic_longitude(jd),
            latitude: get_apparent_ecliptic_latitude(jd),
        }
    }

    pub fn get_apparent_equatorial_coordinates(jd: f64) -> EquatorialCoordinates {
        coordinates::transform_ecliptic_to_equatorial(
            get_apparent_ecliptic_longitude(jd),
            get_apparent_ecliptic_latitude(jd),
            nutation::get_true_obliquity_of_ecliptic(jd),
        )
    }
}

pub mod aberration {
    use super::*;

    pub fn get_ecliptic_aberration(jd: f64, lambda_deg: f64, beta_deg: f64) -> (f64, f64) {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let e = 0.016708634 - 0.000042037 * t - 0.0000001267 * t2;
        let pi = (102.93735 + 1.71946 * t + 0.00046 * t2) * DEG2RAD;
        let k = 20.49552;
        let sun_lng = sun::get_geometric_ecliptic_longitude(jd) * DEG2RAD;
        let lambda = lambda_deg * DEG2RAD;
        let beta = beta_deg * DEG2RAD;

        let x = (-k * (sun_lng - lambda).cos() + e * k * (pi - lambda).cos()) / beta.cos() / 3600.0;
        let y = -k * beta.sin() * ((sun_lng - lambda).sin() - e * (pi - lambda).sin()) / 3600.0;
        (x, y)
    }
}

pub mod elliptical {
    use super::*;

    pub fn get_distance_to_light_time(distance_au: f64) -> f64 {
        distance_au * 0.0057755183
    }

    pub fn get_elliptical_details(
        jd: f64,
        lng_fn: fn(f64) -> f64,
        lat_fn: fn(f64) -> f64,
        rad_fn: fn(f64) -> f64,
    ) -> EllipticalDetails {
        let mut jd0 = jd;
        let l0 = earth::get_ecliptic_longitude(jd0) * DEG2RAD;
        let b0 = earth::get_ecliptic_latitude(jd0) * DEG2RAD;
        let r0 = earth::get_radius_vector(jd0);
        let cos_b0 = b0.cos();

        let mut l = 0.0;
        let mut b = 0.0;
        let mut r = 0.0;

        let mut recalc = true;
        let mut first = true;
        let mut l_prev = 0.0;
        let mut b_prev = 0.0;
        let mut r_prev = 0.0;

        while recalc {
            l = lng_fn(jd0);
            b = lat_fn(jd0);
            r = rad_fn(jd0);

            if !first {
                recalc = (l - l_prev).abs() > 0.00001
                    || (b - b_prev).abs() > 0.00001
                    || (r - r_prev).abs() > 0.000001;
                l_prev = l;
                b_prev = b;
                r_prev = r;
            } else {
                first = false;
                l_prev = l;
                b_prev = b;
                r_prev = r;
            }

            if recalc {
                let l_rad = l * DEG2RAD;
                let b_rad = b * DEG2RAD;
                let cos_b = b_rad.cos();
                let cos_l = l_rad.cos();
                let x = r * cos_b * cos_l - r0 * cos_b0 * l0.cos();
                let y = r * cos_b * l_rad.sin() - r0 * cos_b0 * l0.sin();
                let z = r * b_rad.sin() - r0 * b0.sin();
                let dist = (x * x + y * y + z * z).sqrt();
                jd0 = jd - get_distance_to_light_time(dist);
            }
        }

        let l_rad = l * DEG2RAD;
        let b_rad = b * DEG2RAD;
        let cos_b = b_rad.cos();
        let cos_l = l_rad.cos();
        let x = r * cos_b * cos_l - r0 * cos_b0 * l0.cos();
        let y = r * cos_b * l_rad.sin() - r0 * cos_b0 * l0.sin();
        let z = r * b_rad.sin() - r0 * b0.sin();

        let x2 = x * x;
        let y2 = y * y;
        let apparent_geocentric_distance = (x2 + y2 + z * z).sqrt();
        let apparent_light_time = get_distance_to_light_time(apparent_geocentric_distance);

        let mut lon = map_to_0_360(RAD2DEG * y.atan2(x));
        let mut lat = RAD2DEG * z.atan2((x2 + y2).sqrt());

        let (ab_x, ab_y) = aberration::get_ecliptic_aberration(jd, lon, lat);
        lon += ab_x;
        lat += ab_y;

        let d_long = fk5::get_correction_in_longitude(jd, lon, lat);
        let d_lat = fk5::get_correction_in_latitude(jd, lon);
        lon += d_long;
        lat += d_lat;

        let nut_long = nutation::get_nutation_in_longitude(jd);
        lon += get_decimal(0.0, 0.0, nut_long.abs(), nut_long >= 0.0);
        lon = map_to_0_360(lon);

        let epsilon = nutation::get_true_obliquity_of_ecliptic(jd);
        let apparent_geocentric_equatorial = coordinates::transform_ecliptic_to_equatorial(lon, lat, epsilon);

        EllipticalDetails {
            apparent_light_time,
            apparent_geocentric_distance,
            apparent_geocentric_ecliptic: EclipticCoordinates {
                longitude: lon,
                latitude: lat,
            },
            apparent_geocentric_equatorial,
        }
    }
}

pub mod moon {
    use super::*;
    use crate::coeffs::moon::*;

    pub fn get_mean_longitude(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        map_to_0_360(218.3164477 + 481267.88123421 * t - 0.0015786 * t2 + t3 / 538841.0 - t4 / 65194000.0)
    }

    pub fn get_mean_elongation(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        map_to_0_360(297.8501921 + 445267.1114034 * t - 0.0018819 * t2 + t3 / 545868.0 - t4 / 113065000.0)
    }

    pub fn get_mean_anomaly(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        map_to_0_360(134.9633964 + 477198.8675055 * t + 0.0087414 * t2 + t3 / 69699.0 - t4 / 14712000.0)
    }

    pub fn get_argument_of_latitude(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        map_to_0_360(93.2720950 + 483202.0175233 * t - 0.0036539 * t2 - t3 / 3526000.0 + t4 / 863310000.0)
    }

    pub fn get_ecliptic_longitude(jd: f64) -> f64 {
        let ldash = get_mean_longitude(jd) * DEG2RAD;
        let d = get_mean_elongation(jd) * DEG2RAD;
        let m = earth::get_sun_mean_anomaly(jd) * DEG2RAD;
        let mdash = get_mean_anomaly(jd) * DEG2RAD;
        let f = get_argument_of_latitude(jd) * DEG2RAD;
        let e = earth::get_eccentricity(jd);
        let e2 = e * e;
        let t = (jd - J2000) / 36525.0;

        let a1 = map_to_0_360(119.75 + 131.849 * t) * DEG2RAD;
        let a2 = map_to_0_360(53.09 + 479264.290 * t) * DEG2RAD;

        let mut sigma_l = 0.0;
        for i in 0..gMoonCoefficients1.len() {
            let (cd, cm, cmdash, cf) = gMoonCoefficients1[i];
            let (ca, _) = gMoonCoefficients2[i];
            let mut term = ca * (cd * d + cm * m + cmdash * mdash + cf * f).sin();
            if cm == 1.0 || cm == -1.0 {
                term *= e;
            } else if cm == 2.0 || cm == -2.0 {
                term *= e2;
            }
            sigma_l += term;
        }

        sigma_l += 3958.0 * a1.sin();
        sigma_l += 1962.0 * (ldash - f).sin();
        sigma_l += 318.0 * a2.sin();

        let nut_long = nutation::get_nutation_in_longitude(jd);
        let ldash_deg = ldash * RAD2DEG;
        map_to_0_360(ldash_deg + sigma_l / 1_000_000.0 + nut_long / 3600.0)
    }

    pub fn get_ecliptic_latitude(jd: f64) -> f64 {
        let ldash = get_mean_longitude(jd) * DEG2RAD;
        let d = get_mean_elongation(jd) * DEG2RAD;
        let m = earth::get_sun_mean_anomaly(jd) * DEG2RAD;
        let mdash = get_mean_anomaly(jd) * DEG2RAD;
        let f = get_argument_of_latitude(jd) * DEG2RAD;
        let e = earth::get_eccentricity(jd);
        let e2 = e * e;
        let t = (jd - J2000) / 36525.0;

        let a1 = map_to_0_360(119.75 + 131.849 * t) * DEG2RAD;
        let a3 = map_to_0_360(313.45 + 481266.484 * t) * DEG2RAD;

        let mut sigma_b = 0.0;
        for i in 0..gMoonCoefficients3.len() {
            let (cd, cm, cmdash, cf) = gMoonCoefficients3[i];
            let cb = gMoonCoefficients4[i];
            let mut term = cb * (cd * d + cm * m + cmdash * mdash + cf * f).sin();
            if cm == 1.0 || cm == -1.0 {
                term *= e;
            } else if cm == 2.0 || cm == -2.0 {
                term *= e2;
            }
            sigma_b += term;
        }

        sigma_b -= 2235.0 * ldash.sin();
        sigma_b += 382.0 * a3.sin();
        sigma_b += 175.0 * (a1 - f).sin();
        sigma_b += 175.0 * (a1 + f).sin();
        sigma_b += 127.0 * (ldash - mdash).sin();
        sigma_b -= 115.0 * (ldash + mdash).sin();

        sigma_b / 1_000_000.0
    }

    pub fn get_radius_vector_km(jd: f64) -> f64 {
        let d = get_mean_elongation(jd) * DEG2RAD;
        let m = earth::get_sun_mean_anomaly(jd) * DEG2RAD;
        let mdash = get_mean_anomaly(jd) * DEG2RAD;
        let f = get_argument_of_latitude(jd) * DEG2RAD;
        let e = earth::get_eccentricity(jd);
        let e2 = e * e;

        let mut sigma_r = 0.0;
        for i in 0..gMoonCoefficients1.len() {
            let (cd, cm, cmdash, cf) = gMoonCoefficients1[i];
            let (_, cb) = gMoonCoefficients2[i];
            let mut term = cb * (cd * d + cm * m + cmdash * mdash + cf * f).cos();
            if cm == 1.0 || cm == -1.0 {
                term *= e;
            } else if cm == 2.0 || cm == -2.0 {
                term *= e2;
            }
            sigma_r += term;
        }

        385000.56 + sigma_r / 1000.0
    }

    pub fn get_mean_longitude_ascending_node(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        map_to_0_360(125.0445479 - 1934.1362891 * t + 0.0020754 * t2 + t3 / 467441.0 - t4 / 60616000.0)
    }

    pub fn true_longitude_ascending_node(jd: f64) -> f64 {
        let mut node = get_mean_longitude_ascending_node(jd);
        let d = get_mean_elongation(jd) * DEG2RAD;
        let m = earth::get_sun_mean_anomaly(jd) * DEG2RAD;
        let mdash = get_mean_anomaly(jd) * DEG2RAD;
        let f = get_argument_of_latitude(jd) * DEG2RAD;

        node -= 1.4979 * (2.0 * (d - f)).sin();
        node -= 0.1500 * m.sin();
        node -= 0.1226 * (2.0 * d).sin();
        node += 0.1176 * (2.0 * f).sin();
        node -= 0.0801 * (2.0 * (mdash - f)).sin();

        map_to_0_360(node)
    }

    pub fn get_mean_longitude_perigee(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        map_to_0_360(83.3532465 + 4069.0137287 * t - 0.0103200 * t2 - t3 / 80053.0 + t4 / 18999000.0)
    }

    /// Lilith Média (Apogeu Lunar Médio = Perigeu + 180°)
    pub fn get_mean_lilith_apogee(jd: f64) -> f64 {
        map_to_0_360(get_mean_longitude_perigee(jd) + 180.0)
    }

    pub fn get_equatorial_coordinates(jd: f64) -> EquatorialCoordinates {
        coordinates::transform_ecliptic_to_equatorial(
            get_ecliptic_longitude(jd),
            get_ecliptic_latitude(jd),
            nutation::get_true_obliquity_of_ecliptic(jd),
        )
    }
}

macro_rules! impl_vsop_planet {
    ($mod_name:ident, $coeff_mod:ident, [$($l_arr:ident),+], [$($b_arr:ident),+], [$($r_arr:ident),+]) => {
        pub mod $mod_name {
            use super::*;
            use crate::coeffs::$coeff_mod::*;

            pub fn get_ecliptic_longitude(jd: f64) -> f64 {
                let rho = (jd - J2000) / 365250.0;
                let mut rho_pow = 1.0;
                let mut total = 0.0;
                $(
                    total += sum_vsop($l_arr, rho) * rho_pow;
                    rho_pow *= rho;
                )+
                let _ = rho_pow;
                map_to_0_360(RAD2DEG * (total / 100_000_000.0))
            }

            pub fn get_ecliptic_latitude(jd: f64) -> f64 {
                let rho = (jd - J2000) / 365250.0;
                let mut rho_pow = 1.0;
                let mut total = 0.0;
                $(
                    total += sum_vsop($b_arr, rho) * rho_pow;
                    rho_pow *= rho;
                )+
                let _ = rho_pow;
                map_to_minus90_90(RAD2DEG * (total / 100_000_000.0))
            }

            pub fn get_radius_vector(jd: f64) -> f64 {
                let rho = (jd - J2000) / 365250.0;
                let mut rho_pow = 1.0;
                let mut total = 0.0;
                $(
                    total += sum_vsop($r_arr, rho) * rho_pow;
                    rho_pow *= rho;
                )+
                let _ = rho_pow;
                total / 100_000_000.0
            }

            pub fn get_planetary_details(jd: f64) -> EllipticalDetails {
                elliptical::get_elliptical_details(
                    jd,
                    get_ecliptic_longitude,
                    get_ecliptic_latitude,
                    get_radius_vector,
                )
            }
        }
    };
}

impl_vsop_planet!(
    mercury, mercury,
    [g_L0MercuryCoefficients, g_L1MercuryCoefficients, g_L2MercuryCoefficients, g_L3MercuryCoefficients, g_L4MercuryCoefficients, g_L5MercuryCoefficients],
    [g_B0MercuryCoefficients, g_B1MercuryCoefficients, g_B2MercuryCoefficients, g_B3MercuryCoefficients, g_B4MercuryCoefficients],
    [g_R0MercuryCoefficients, g_R1MercuryCoefficients, g_R2MercuryCoefficients, g_R3MercuryCoefficients]
);

impl_vsop_planet!(
    venus, venus,
    [g_L0VenusCoefficients, g_L1VenusCoefficients, g_L2VenusCoefficients, g_L3VenusCoefficients, g_L4VenusCoefficients, g_L5VenusCoefficients],
    [g_B0VenusCoefficients, g_B1VenusCoefficients, g_B2VenusCoefficients, g_B3VenusCoefficients, g_B4VenusCoefficients],
    [g_R0VenusCoefficients, g_R1VenusCoefficients, g_R2VenusCoefficients, g_R3VenusCoefficients]
);

impl_vsop_planet!(
    mars, mars,
    [g_L0MarsCoefficients, g_L1MarsCoefficients, g_L2MarsCoefficients, g_L3MarsCoefficients, g_L4MarsCoefficients, g_L5MarsCoefficients],
    [g_B0MarsCoefficients, g_B1MarsCoefficients, g_B2MarsCoefficients, g_B3MarsCoefficients, g_B4MarsCoefficients],
    [g_R0MarsCoefficients, g_R1MarsCoefficients, g_R2MarsCoefficients, g_R3MarsCoefficients, g_R4MarsCoefficients]
);

impl_vsop_planet!(
    jupiter, jupiter,
    [g_L0JupiterCoefficients, g_L1JupiterCoefficients, g_L2JupiterCoefficients, g_L3JupiterCoefficients, g_L4JupiterCoefficients, g_L5JupiterCoefficients],
    [g_B0JupiterCoefficients, g_B1JupiterCoefficients, g_B2JupiterCoefficients, g_B3JupiterCoefficients, g_B4JupiterCoefficients, g_B5JupiterCoefficients],
    [g_R0JupiterCoefficients, g_R1JupiterCoefficients, g_R2JupiterCoefficients, g_R3JupiterCoefficients, g_R4JupiterCoefficients, g_R5JupiterCoefficients]
);

impl_vsop_planet!(
    saturn, saturn,
    [g_L0SaturnCoefficients, g_L1SaturnCoefficients, g_L2SaturnCoefficients, g_L3SaturnCoefficients, g_L4SaturnCoefficients, g_L5SaturnCoefficients],
    [g_B0SaturnCoefficients, g_B1SaturnCoefficients, g_B2SaturnCoefficients, g_B3SaturnCoefficients, g_B4SaturnCoefficients, g_B5SaturnCoefficients],
    [g_R0SaturnCoefficients, g_R1SaturnCoefficients, g_R2SaturnCoefficients, g_R3SaturnCoefficients, g_R4SaturnCoefficients, g_R5SaturnCoefficients]
);

impl_vsop_planet!(
    uranus, uranus,
    [g_L0UranusCoefficients, g_L1UranusCoefficients, g_L2UranusCoefficients, g_L3UranusCoefficients, g_L4UranusCoefficients],
    [g_B0UranusCoefficients, g_B1UranusCoefficients, g_B2UranusCoefficients, g_B3UranusCoefficients, g_B4UranusCoefficients],
    [g_R0UranusCoefficients, g_R1UranusCoefficients, g_R2UranusCoefficients, g_R3UranusCoefficients, g_R4UranusCoefficients]
);

impl_vsop_planet!(
    neptune, neptune,
    [g_L0NeptuneCoefficients, g_L1NeptuneCoefficients, g_L2NeptuneCoefficients, g_L3NeptuneCoefficients, g_L4NeptuneCoefficients],
    [g_B0NeptuneCoefficients, g_B1NeptuneCoefficients, g_B2NeptuneCoefficients, g_B3NeptuneCoefficients, g_B4NeptuneCoefficients],
    [g_R0NeptuneCoefficients, g_R1NeptuneCoefficients, g_R2NeptuneCoefficients, g_R3NeptuneCoefficients]
);

pub mod pluto {
    use super::*;
    use crate::coeffs::pluto::*;

    pub fn get_ecliptic_longitude(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let j = 34.35 + 3034.9057 * t;
        let s = 50.08 + 1222.1138 * t;
        let p = 238.96 + 144.9600 * t;

        let mut l = 0.0;
        for i in 0..g_PlutoArgumentCoefficients.len() {
            let (cj, cs, cp) = g_PlutoArgumentCoefficients[i];
            let (ca, cb) = g_PlutoLongitudeCoefficients[i];
            let alpha = (cj * j + cs * s + cp * p) * DEG2RAD;
            l += ca * alpha.sin() + cb * alpha.cos();
        }
        l = l / 1_000_000.0 + (238.958116 + 144.96 * t);
        map_to_0_360(l)
    }

    pub fn get_ecliptic_latitude(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let j = 34.35 + 3034.9057 * t;
        let s = 50.08 + 1222.1138 * t;
        let p = 238.96 + 144.9600 * t;

        let mut b = 0.0;
        for i in 0..g_PlutoArgumentCoefficients.len() {
            let (cj, cs, cp) = g_PlutoArgumentCoefficients[i];
            let (ca, cb) = g_PlutoLatitudeCoefficients[i];
            let alpha = (cj * j + cs * s + cp * p) * DEG2RAD;
            b += ca * alpha.sin() + cb * alpha.cos();
        }
        b = b / 1_000_000.0 - 3.908239;
        map_to_minus90_90(b)
    }

    pub fn get_radius_vector(jd: f64) -> f64 {
        let t = (jd - J2000) / 36525.0;
        let j = 34.35 + 3034.9057 * t;
        let s = 50.08 + 1222.1138 * t;
        let p = 238.96 + 144.9600 * t;

        let mut r = 0.0;
        for i in 0..g_PlutoArgumentCoefficients.len() {
            let (cj, cs, cp) = g_PlutoArgumentCoefficients[i];
            let (ca, cb) = g_PlutoRadiusCoefficients[i];
            let alpha = (cj * j + cs * s + cp * p) * DEG2RAD;
            r += ca * alpha.sin() + cb * alpha.cos();
        }
        r / 10_000_000.0 + 40.7241346
    }

    pub fn get_planetary_details(jd: f64) -> EllipticalDetails {
        elliptical::get_elliptical_details(
            jd,
            get_ecliptic_longitude,
            get_ecliptic_latitude,
            get_radius_vector,
        )
    }
}

pub mod chiron {
    use super::*;

    /// Tabela multi-época de elementos orbitais osculadores de 2060 Quíron (Chiron)
    /// Referencial Eclíptico J2000.0 (JPL Horizons / MPC):
    /// (jd_epoch, a_au, e, incl_deg, node_deg, arg_peri_deg, mean_anom_deg)
    const CHIRON_EPOCHS: &[(f64, f64, f64, f64, f64, f64, f64)] = &[
        (2415020.5, 13.68200, 0.38050, 6.9240, 209.6800, 339.1200, 319.8500), // 1900-01-01
        (2433282.5, 13.64500, 0.37860, 6.9220, 209.6150, 339.4200,  33.1800), // 1950-01-01
        (2443448.5, 13.69480, 0.37860, 6.9230, 209.5910, 339.5050, 230.1250), // 1977-11-01
        (2450127.5, 13.69540, 0.38200, 6.9310, 209.3920, 339.5600,   0.0000), // 1996-02-14 (Periélio)
        (2451545.0, 13.69619, 0.38209, 6.9310, 209.3906, 339.5613,  27.5620), // 2000-01-01.5 (J2000)
        (2459000.5, 13.70250, 0.37925, 6.9295, 209.3580, 339.9400, 172.3400), // 2020-05-31
        (2460800.5, 13.70420, 0.37911, 6.9292, 209.3550, 339.9620, 207.3300), // 2025-05-05
        (2469807.5, 13.69800, 0.38120, 6.9300, 209.3400, 339.8800,  22.3900), // 2050-01-01
    ];

    fn helio_ecliptic_coords(jd: f64) -> (f64, f64, f64) {
        // Seleciona a época osculadora mais próxima de jd
        let mut best = CHIRON_EPOCHS[0];
        let mut min_dist = (jd - best.0).abs();
        for &ep in &CHIRON_EPOCHS[1..] {
            let d = (jd - ep.0).abs();
            if d < min_dist {
                min_dist = d;
                best = ep;
            }
        }
        let (jd0, a, e, i_deg, node_deg, argp_deg, m0_deg) = best;

        // Movimento médio diário gaussiano (k = 0.9856076686°/dia)
        let n_deg_per_day = 0.9856076686 / (a * a.sqrt());
        let m_deg = map_to_0_360(m0_deg + n_deg_per_day * (jd - jd0));
        let m_rad = m_deg * DEG2RAD;

        // Resolução da Equação de Kepler: E - e * sin(E) = M (Newton-Raphson)
        let mut e_anom = m_rad + e * m_rad.sin() * (1.0 + e * m_rad.cos());
        for _ in 0..8 {
            let f = e_anom - e * e_anom.sin() - m_rad;
            let f_prime = 1.0 - e * e_anom.cos();
            let delta = f / f_prime;
            e_anom -= delta;
            if delta.abs() < 1e-12 {
                break;
            }
        }

        // Coordenadas no plano orbital e vetor raio heliocêntrico (UA)
        let xv = a * (e_anom.cos() - e);
        let yv = a * (1.0 - e * e).sqrt() * e_anom.sin();
        let v_rad = yv.atan2(xv);
        let r_au = (xv * xv + yv * yv).sqrt();

        // Transformação para a eclíptica J2000 e precessão até a eclíptica da data
        let u_rad = argp_deg * DEG2RAD + v_rad;
        let i_rad = i_deg * DEG2RAD;
        let node_rad = node_deg * DEG2RAD;

        let sin_u = u_rad.sin();
        let cos_u = u_rad.cos();
        let sin_i = i_rad.sin();
        let cos_i = i_rad.cos();

        let lon_j2000 = node_rad + (sin_u * cos_i).atan2(cos_u);
        let lat_j2000 = (sin_u * sin_i).asin();

        let t_cent = (jd - J2000) / 36525.0;
        let precess_deg = 1.396971 * t_cent + 0.0003086 * t_cent * t_cent;

        let lon_deg = map_to_0_360(lon_j2000 * RAD2DEG + precess_deg);
        let lat_deg = map_to_minus90_90(lat_j2000 * RAD2DEG);

        (lon_deg, lat_deg, r_au)
    }

    pub fn get_ecliptic_longitude(jd: f64) -> f64 {
        helio_ecliptic_coords(jd).0
    }

    pub fn get_ecliptic_latitude(jd: f64) -> f64 {
        helio_ecliptic_coords(jd).1
    }

    pub fn get_radius_vector(jd: f64) -> f64 {
        helio_ecliptic_coords(jd).2
    }

    pub fn get_planetary_details(jd: f64) -> EllipticalDetails {
        elliptical::get_elliptical_details(
            jd,
            get_ecliptic_longitude,
            get_ecliptic_latitude,
            get_radius_vector,
        )
    }
}

// ============================================================================
// CAMADA DE APLICAÇÃO ASTROLÓGICA / ASTRONÔMICA (Signos, Graus, Velocidade, Rx)
// ============================================================================

pub const ZODIAC_SIGNS: [(&str, &str, &str); 12] = [
    ("Áries", "♈", "Fogo"),
    ("Touro", "♉", "Terra"),
    ("Gêmeos", "♊", "Ar"),
    ("Câncer", "♋", "Água"),
    ("Leão", "♌", "Fogo"),
    ("Virgem", "♍", "Terra"),
    ("Libra", "♎", "Ar"),
    ("Escorpião", "♏", "Água"),
    ("Sagitário", "♐", "Fogo"),
    ("Capricórnio", "♑", "Terra"),
    ("Aquário", "♒", "Ar"),
    ("Peixes", "♓", "Água"),
];

#[derive(Debug, Clone)]
pub struct AstroBodyResult {
    pub id: &'static str,
    pub name: &'static str,
    pub symbol: &'static str,
    pub category: &'static str,
    pub longitude: f64,
    pub latitude: f64,
    pub right_ascension: f64,
    pub declination: f64,
    pub distance: f64,
    pub distance_unit: &'static str,
    pub daily_speed: f64,
    pub is_retrograde: bool,
    pub sign_index: usize,
    pub sign_name: &'static str,
    pub sign_symbol: &'static str,
    pub element: &'static str,
    pub degree_in_sign: i32,
    pub minutes: i32,
    pub seconds: f64,
    pub formatted_position: String,
}

fn diff_angle_deg(lon2: f64, lon1: f64) -> f64 {
    let mut d = lon2 - lon1;
    if d > 180.0 {
        d -= 360.0;
    } else if d < -180.0 {
        d += 360.0;
    }
    d
}

fn build_body_result(
    id: &'static str,
    name: &'static str,
    symbol: &'static str,
    category: &'static str,
    longitude: f64,
    latitude: f64,
    right_ascension: f64,
    declination: f64,
    distance: f64,
    distance_unit: &'static str,
    daily_speed: f64,
) -> AstroBodyResult {
    let lon = map_to_0_360(longitude);
    let sign_index = ((lon / 30.0).floor() as usize).min(11);
    let (sign_name, sign_symbol, element) = ZODIAC_SIGNS[sign_index];
    let deg_in_sign_dec = lon - (sign_index as f64) * 30.0;
    let dms = get_sexagesimal(deg_in_sign_dec);
    let is_retrograde = daily_speed < 0.0;
    let rx_tag = if is_retrograde { " Rx" } else { "" };
    let formatted_position = format!(
        "{:02}° {} {:02}' {:04.1}"{}",
        dms.radix, sign_symbol, dms.minutes, dms.seconds, rx_tag
    );

    AstroBodyResult {
        id,
        name,
        symbol,
        category,
        longitude: lon,
        latitude,
        right_ascension,
        declination,
        distance,
        distance_unit,
        daily_speed,
        is_retrograde,
        sign_index,
        sign_name,
        sign_symbol,
        element,
        degree_in_sign: dms.radix,
        minutes: dms.minutes,
        seconds: dms.seconds,
        formatted_position,
    }
}

pub fn calculate_all_bodies(jd: f64) -> Vec<AstroBodyResult> {
    let dt = 0.005; // ~7.2 minutos para derivada numérica central de altíssima precisão
    let mut results = Vec::with_capacity(16);

    // 1. Sol
    {
        let ecl = sun::get_apparent_ecliptic_coordinates(jd);
        let equ = sun::get_apparent_equatorial_coordinates(jd);
        let dist = earth::get_radius_vector(jd);
        let lon_prev = sun::get_apparent_ecliptic_longitude(jd - dt);
        let lon_next = sun::get_apparent_ecliptic_longitude(jd + dt);
        let speed = diff_angle_deg(lon_next, lon_prev) / (2.0 * dt);
        results.push(build_body_result(
            "sun", "Sol", "☉", "Luminar",
            ecl.longitude, ecl.latitude, equ.right_ascension, equ.declination,
            dist, "UA", speed,
        ));
    }

    // 2. Lua
    {
        let lon = moon::get_ecliptic_longitude(jd);
        let lat = moon::get_ecliptic_latitude(jd);
        let equ = moon::get_equatorial_coordinates(jd);
        let dist = moon::get_radius_vector_km(jd);
        let lon_prev = moon::get_ecliptic_longitude(jd - dt);
        let lon_next = moon::get_ecliptic_longitude(jd + dt);
        let speed = diff_angle_deg(lon_next, lon_prev) / (2.0 * dt);
        results.push(build_body_result(
            "moon", "Lua", "☽", "Luminar",
            lon, lat, equ.right_ascension, equ.declination,
            dist, "km", speed,
        ));
    }

    // 3..11. Planetas (Mercúrio a Plutão) + Quíron (Centauro / Asteroide 2060)
    let planet_specs: [(&str, &str, &str, &str, fn(f64) -> EllipticalDetails); 9] = [
        ("mercury", "Mercúrio", "☿", "Pessoal", mercury::get_planetary_details),
        ("venus", "Vênus", "♀", "Pessoal", venus::get_planetary_details),
        ("mars", "Marte", "♂", "Pessoal", mars::get_planetary_details),
        ("jupiter", "Júpiter", "♃", "Social", jupiter::get_planetary_details),
        ("saturn", "Saturno", "♄", "Social", saturn::get_planetary_details),
        ("chiron", "Quíron", "⚷", "Centauro", chiron::get_planetary_details),
        ("uranus", "Urano", "♅", "Geracional", uranus::get_planetary_details),
        ("neptune", "Netuno", "♆", "Geracional", neptune::get_planetary_details),
        ("pluto", "Plutão", "♇", "Geracional", pluto::get_planetary_details),
    ];

    for (id, name, symbol, category, details_fn) in planet_specs {
        let det = details_fn(jd);
        let det_prev = details_fn(jd - dt);
        let det_next = details_fn(jd + dt);
        let speed = diff_angle_deg(
            det_next.apparent_geocentric_ecliptic.longitude,
            det_prev.apparent_geocentric_ecliptic.longitude,
        ) / (2.0 * dt);

        results.push(build_body_result(
            id,
            name,
            symbol,
            category,
            det.apparent_geocentric_ecliptic.longitude,
            det.apparent_geocentric_ecliptic.latitude,
            det.apparent_geocentric_equatorial.right_ascension,
            det.apparent_geocentric_equatorial.declination,
            det.apparent_geocentric_distance,
            "UA",
            speed,
        ));
    }

    // 11. Nodo Norte Médio (Cabeça do Dragão — padrão astrológico tradicional/moderno)
    {
        let eps = nutation::get_true_obliquity_of_ecliptic(jd);
        let lon = moon::get_mean_longitude_ascending_node(jd);
        let lon_prev = moon::get_mean_longitude_ascending_node(jd - dt);
        let lon_next = moon::get_mean_longitude_ascending_node(jd + dt);
        let speed = diff_angle_deg(lon_next, lon_prev) / (2.0 * dt);
        let equ = coordinates::transform_ecliptic_to_equatorial(lon, 0.0, eps);
        results.push(build_body_result(
            "mean_north_node", "Nodo Norte (Médio)", "☊", "Ponto Lunar",
            lon, 0.0, equ.right_ascension, equ.declination,
            0.0, "-", speed,
        ));

        // 12. Nodo Sul Médio (Cauda do Dragão = Nodo Norte Médio + 180°)
        let south_lon = map_to_0_360(lon + 180.0);
        let equ_s = coordinates::transform_ecliptic_to_equatorial(south_lon, 0.0, eps);
        results.push(build_body_result(
            "mean_south_node", "Nodo Sul (Médio)", "☋", "Ponto Lunar",
            south_lon, 0.0, equ_s.right_ascension, equ_s.declination,
            0.0, "-", speed,
        ));
    }

    // 13. Nodo Norte Verdadeiro (Cabeça do Dragão — corrigido por perturbações periódicas)
    {
        let eps = nutation::get_true_obliquity_of_ecliptic(jd);
        let lon = moon::true_longitude_ascending_node(jd);
        let lon_prev = moon::true_longitude_ascending_node(jd - dt);
        let lon_next = moon::true_longitude_ascending_node(jd + dt);
        let speed = diff_angle_deg(lon_next, lon_prev) / (2.0 * dt);
        let equ = coordinates::transform_ecliptic_to_equatorial(lon, 0.0, eps);
        results.push(build_body_result(
            "true_north_node", "Nodo Norte (Verdadeiro)", "☊", "Ponto Lunar",
            lon, 0.0, equ.right_ascension, equ.declination,
            0.0, "-", speed,
        ));

        // 14. Nodo Sul Verdadeiro (Cauda do Dragão = Nodo Norte Verdadeiro + 180°)
        let south_lon = map_to_0_360(lon + 180.0);
        let equ_s = coordinates::transform_ecliptic_to_equatorial(south_lon, 0.0, eps);
        results.push(build_body_result(
            "true_south_node", "Nodo Sul (Verdadeiro)", "☋", "Ponto Lunar",
            south_lon, 0.0, equ_s.right_ascension, equ_s.declination,
            0.0, "-", speed,
        ));
    }

    // 15. Lilith Média (Lua Negra - Apogeu Lunar Médio)
    {
        let eps = nutation::get_true_obliquity_of_ecliptic(jd);
        let lon = moon::get_mean_lilith_apogee(jd);
        let lon_prev = moon::get_mean_lilith_apogee(jd - dt);
        let lon_next = moon::get_mean_lilith_apogee(jd + dt);
        let speed = diff_angle_deg(lon_next, lon_prev) / (2.0 * dt);
        let equ = coordinates::transform_ecliptic_to_equatorial(lon, 0.0, eps);
        results.push(build_body_result(
            "mean_lilith", "Lilith (Lua Negra Média)", "⚸", "Ponto Lunar",
            lon, 0.0, equ.right_ascension, equ.declination,
            0.0, "-", speed,
        ));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moon_coordinates_meeus_1992() {
        // Exemplo clássico do livro de Jean Meeus (12 de Abril de 1992 00:00 UTC)
        let jd = julianday::from_utc(1992, 4, 12, 0, 0, 0.0);
        assert!((jd - 2448724.5).abs() < 1e-6);
        let lon = moon::get_ecliptic_longitude(jd);
        let lat = moon::get_ecliptic_latitude(jd);
        assert!((lon - 133.167265).abs() < 0.01);
        assert!((lat - (-3.229126)).abs() < 0.01);
    }

    #[test]
    fn test_venus_elliptical_meeus_1992() {
        // Exemplo 33.a de Jean Meeus (20 de Dezembro de 1992 00:00 TD -> JD 2448976.5)
        let jd = 2448976.5;
        let details = venus::get_planetary_details(jd);
        assert!((details.apparent_geocentric_ecliptic.longitude - 313.0815).abs() < 0.01);
        assert!((details.apparent_geocentric_distance - 0.910845).abs() < 0.001);
    }

    #[test]
    fn test_mean_and_true_lunar_nodes() {
        let jd = julianday::from_utc(2026, 10, 30, 17, 0, 0.0);
        let mean_nn = moon::get_mean_longitude_ascending_node(jd);
        let true_nn = moon::true_longitude_ascending_node(jd);
        // O Nodo Médio e o Nodo Verdadeiro devem diferir em no máximo ~1.8°
        let diff = (mean_nn - true_nn).abs();
        assert!(diff < 2.0);
        let bodies = calculate_all_bodies(jd);
        assert!(bodies.iter().any(|b| b.id == "chiron"));
        assert!(bodies.iter().any(|b| b.id == "mean_north_node"));
        assert!(bodies.iter().any(|b| b.id == "mean_south_node"));
        assert!(bodies.iter().any(|b| b.id == "true_north_node"));
        assert!(bodies.iter().any(|b| b.id == "true_south_node"));
    }
}
