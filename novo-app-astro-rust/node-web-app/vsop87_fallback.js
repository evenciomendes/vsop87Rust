const fs = require('fs');
const path = require('path');
const COEFFS = JSON.parse(fs.readFileSync(path.join(__dirname, 'coeffs_data.json'), 'utf8'));

const J1970 = 2440588.0;
const J2000 = 2451545.0;
const DAYMS = 86400000.0;
const DEG2RAD = Math.PI / 180.0;
const RAD2DEG = 180.0 / Math.PI;
const DEG2H = 1.0 / 15.0;

function mapTo0To360(deg) {
  let r = deg % 360.0;
  if (r < 0) r += 360.0;
  return r;
}

function mapToMinus90To90(deg) {
  let r = mapTo0To360(deg);
  if (r > 270.0) r -= 360.0;
  else if (r > 90.0) r = 180.0 - r;
  return r;
}

function getDecimal(d, m, s, positive = true) {
  const val = Math.abs(d) + Math.abs(m) / 60.0 + Math.abs(s) / 3600.0;
  return positive ? val : -val;
}

function getSexagesimal(decimal) {
  const absVal = Math.abs(decimal);
  const radix = Math.floor(absVal);
  const fracDeg = absVal - radix;
  const fracMin = fracDeg * 60.0;
  const minutes = Math.floor(fracMin);
  const seconds = (fracMin - minutes) * 60.0;
  return { radix, minutes, seconds };
}

function sumVsop(arr, rho) {
  let s = 0.0;
  for (let i = 0; i < arr.length; i++) {
    const row = arr[i];
    s += row[0] * Math.cos(row[1] + row[2] * rho);
  }
  return s;
}

const NUTATION_COEFFS = [
  [0,0,0,0,1,-171996,-174.2,92025,8.9],[-2,0,0,2,2,-13187,-1.6,5736,-3.1],[0,0,0,2,2,-2274,-0.2,977,-0.5],
  [0,0,0,0,2,2062,0.2,-895,0.5],[0,1,0,0,0,1426,-3.4,54,-0.1],[0,0,1,0,0,712,0.1,-7,0],
  [-2,1,0,2,2,-517,1.2,224,-0.6],[0,0,0,2,1,-386,-0.4,200,0],[0,0,1,2,2,-301,0,129,-0.1],
  [-2,-1,0,2,2,217,-0.5,-95,0.3],[-2,0,1,0,0,-158,0,0,0],[-2,0,0,2,1,129,0.1,-70,0],
  [0,0,-1,2,2,123,0,-53,0],[2,0,0,0,0,63,0,0,0],[0,0,1,0,1,63,0.1,-33,0],
  [2,0,-1,2,2,-59,0,26,0],[0,0,-1,0,1,-58,-0.1,32,0],[0,0,1,2,1,-51,0,27,0],
  [-2,0,2,0,0,48,0,0,0],[0,0,-2,2,1,46,0,-24,0],[2,0,0,2,2,-38,0,16,0],
  [0,0,2,2,2,-31,0,13,0],[0,0,2,0,0,29,0,0,0],[-2,0,1,2,2,29,0,-12,0],
  [0,0,0,2,0,26,0,0,0],[-2,0,0,2,0,-22,0,0,0],[0,0,-1,2,1,21,0,-10,0],
  [0,2,0,0,0,17,-0.1,0,0],[2,0,-1,0,1,16,0,-8,0],[-2,2,0,2,2,-16,0.1,7,0],
  [0,1,0,0,1,-15,0,9,0],[-2,0,1,0,1,-13,0,7,0],[0,-1,0,0,1,-12,0,6,0],
  [0,0,2,-2,0,11,0,0,0],[2,0,-1,2,1,-10,0,5,0],[2,0,1,2,2,-8,0,3,0],
  [0,1,0,2,2,7,0,-3,0],[-2,1,1,0,0,-7,0,0,0],[0,-1,0,2,2,-7,0,3,0],
  [2,0,0,2,1,-7,0,3,0],[2,0,1,0,0,6,0,0,0],[-2,0,2,2,2,6,0,-3,0],
  [-2,0,1,2,1,6,0,-3,0],[2,0,-2,0,1,-6,0,3,0],[2,0,0,0,1,-6,0,3,0],
  [0,-1,1,0,0,5,0,0,0],[-2,-1,0,2,1,-5,0,3,0],[-2,0,0,0,1,-5,0,3,0],
  [0,0,2,2,1,-5,0,3,0],[-2,0,2,0,1,4,0,0,0],[-2,1,0,2,1,4,0,0,0],
  [0,0,1,-2,0,4,0,0,0],[-1,0,1,0,0,-4,0,0,0],[-2,1,0,0,0,-4,0,0,0],
  [1,0,0,0,0,-4,0,0,0],[0,0,1,2,0,3,0,0,0],[0,0,-2,2,2,-3,0,0,0],
  [-1,-1,1,0,0,-3,0,0,0],[0,1,1,0,0,-3,0,0,0],[0,-1,1,2,2,-3,0,0,0],
  [2,-1,-1,2,2,-3,0,0,0],[0,0,3,2,2,-3,0,0,0],[2,-1,0,2,2,-3,0,0,0]
];

function nutationArgs(jd) {
  const T = (jd - J2000) / 36525.0;
  const T2 = T * T, T3 = T2 * T;
  const D = mapTo0To360(297.85036 + 445267.111480 * T - 0.0019142 * T2 + T3 / 189474.0);
  const M = mapTo0To360(357.52772 + 35999.050340 * T - 0.0001603 * T2 - T3 / 300000.0);
  const Mprime = mapTo0To360(134.96298 + 477198.867398 * T + 0.0086972 * T2 + T3 / 56250.0);
  const F = mapTo0To360(93.27191 + 483202.017538 * T - 0.0036825 * T2 + T3 / 327270.0);
  const omega = mapTo0To360(125.04452 - 1934.136261 * T + 0.0020708 * T2 + T3 / 450000.0);
  return { T, D, M, Mprime, F, omega };
}

function getNutationInLongitude(jd) {
  const { T, D, M, Mprime, F, omega } = nutationArgs(jd);
  let val = 0;
  for (const c of NUTATION_COEFFS) {
    const arg = c[0]*D + c[1]*M + c[2]*Mprime + c[3]*F + c[4]*omega;
    val += (c[5] + c[6]*T) * Math.sin(arg * DEG2RAD) * 0.0001;
  }
  return val;
}

function getNutationInObliquity(jd) {
  const { T, D, M, Mprime, F, omega } = nutationArgs(jd);
  let val = 0;
  for (const c of NUTATION_COEFFS) {
    const arg = c[0]*D + c[1]*M + c[2]*Mprime + c[3]*F + c[4]*omega;
    val += (c[7] + c[8]*T) * Math.cos(arg * DEG2RAD) * 0.0001;
  }
  return val;
}

function getMeanObliquity(jd) {
  const U = (jd - J2000) / 3652500.0;
  return getDecimal(23, 26, 21.448)
    - getDecimal(0, 0, 4680.93) * U
    - getDecimal(0, 0, 1.55) * (U**2)
    + getDecimal(0, 0, 1999.25) * (U**3)
    - getDecimal(0, 0, 51.38) * (U**4)
    - getDecimal(0, 0, 249.67) * (U**5)
    - getDecimal(0, 0, 39.05) * (U**6)
    + getDecimal(0, 0, 7.12) * (U**7)
    + getDecimal(0, 0, 27.87) * (U**8)
    + getDecimal(0, 0, 5.79) * (U**9)
    + getDecimal(0, 0, 2.45) * (U**10);
}

function getTrueObliquity(jd) {
  const n = getNutationInObliquity(jd);
  return getMeanObliquity(jd) + getDecimal(0, 0, Math.abs(n), n >= 0);
}

function fk5Long(jd, lng, lat) {
  const T = (jd - J2000) / 36525.0;
  const Ldash = lng - 1.397 * T - 0.00031 * T * T;
  const val = -0.09033 + 0.03916 * (Math.cos(Ldash * DEG2RAD) + Math.sin(Ldash * DEG2RAD)) * Math.tan(lat * DEG2RAD);
  return getDecimal(0, 0, Math.abs(val), val >= 0);
}

function fk5Lat(jd, lng) {
  const T = (jd - J2000) / 36525.0;
  const Ldash = lng - 1.397 * T - 0.00031 * T * T;
  const val = 0.03916 * (Math.cos(Ldash * DEG2RAD) - Math.sin(Ldash * DEG2RAD));
  return getDecimal(0, 0, Math.abs(val), val >= 0);
}

function eclipticToEquatorial(l, b, eps) {
  const dec = Math.asin(Math.sin(b * DEG2RAD) * Math.cos(eps * DEG2RAD) + Math.cos(b * DEG2RAD) * Math.sin(eps * DEG2RAD) * Math.sin(l * DEG2RAD)) * RAD2DEG;
  const y = Math.sin(l * DEG2RAD) * Math.cos(eps * DEG2RAD) - Math.tan(b * DEG2RAD) * Math.sin(eps * DEG2RAD);
  const x = Math.cos(l * DEG2RAD);
  let ra = (Math.atan2(y, x) * RAD2DEG * DEG2H) % 24.0;
  if (ra < 0) ra += 24.0;
  return { rightAscension: ra, declination: dec };
}

function earthLon(jd) {
  const rho = (jd - J2000) / 365250.0;
  const c = COEFFS.earth;
  const val = (sumVsop(c.gL0EarthCoefficients, rho)
    + sumVsop(c.gL1EarthCoefficients, rho) * rho
    + sumVsop(c.gL2EarthCoefficients, rho) * (rho**2)
    + sumVsop(c.gL3EarthCoefficients, rho) * (rho**3)
    + sumVsop(c.gL4EarthCoefficients, rho) * (rho**4)
    + sumVsop(c.gL5EarthCoefficients, rho) * (rho**5)) / 1e8;
  return mapTo0To360(val * RAD2DEG);
}

function earthLat(jd) {
  const rho = (jd - J2000) / 365250.0;
  const c = COEFFS.earth;
  const val = (sumVsop(c.gB0EarthCoefficients, rho) + sumVsop(c.gB1EarthCoefficients, rho) * rho) / 1e8;
  return mapToMinus90To90(val * RAD2DEG);
}

function earthRad(jd) {
  const rho = (jd - J2000) / 365250.0;
  const c = COEFFS.earth;
  return (sumVsop(c.gR0EarthCoefficients, rho)
    + sumVsop(c.gR1EarthCoefficients, rho) * rho
    + sumVsop(c.gR2EarthCoefficients, rho) * (rho**2)
    + sumVsop(c.gR3EarthCoefficients, rho) * (rho**3)
    + sumVsop(c.gR4EarthCoefficients, rho) * (rho**4)) / 1e8;
}

function sunApparentLon(jd) {
  let lon = mapTo0To360(earthLon(jd) + 180.0);
  const lat = -earthLat(jd);
  lon += fk5Long(jd, lon, lat);
  const nut = getNutationInLongitude(jd);
  lon += getDecimal(0, 0, Math.abs(nut), nut >= 0);
  lon -= getDecimal(0, 0, 20.4898 / earthRad(jd), true);
  return mapTo0To360(lon);
}

function sunApparentLat(jd) {
  const lon = mapTo0To360(earthLon(jd) + 180.0);
  let lat = -earthLat(jd);
  lat += fk5Lat(jd, lon);
  return lat;
}

function eclipticAberration(jd, lambdaDeg, betaDeg) {
  const T = (jd - J2000) / 36525.0;
  const e = 0.016708634 - 0.000042037 * T - 0.0000001267 * T * T;
  const pi = (102.93735 + 1.71946 * T + 0.00046 * T * T) * DEG2RAD;
  const k = 20.49552;
  const sunLng = mapTo0To360(earthLon(jd) + 180.0) * DEG2RAD;
  const lam = lambdaDeg * DEG2RAD;
  const bet = betaDeg * DEG2RAD;
  return {
    X: (-k * Math.cos(sunLng - lam) + e * k * Math.cos(pi - lam)) / Math.cos(bet) / 3600.0,
    Y: -k * Math.sin(bet) * (Math.sin(sunLng - lam) - e * Math.sin(pi - lam)) / 3600.0
  };
}

function evalVsopPlanet(planetKey, lKeys, bKeys, rKeys, jd) {
  const rho = (jd - J2000) / 365250.0;
  const c = COEFFS[planetKey];
  let lSum = 0, rhoP = 1;
  for (const k of lKeys) { lSum += sumVsop(c[k], rho) * rhoP; rhoP *= rho; }
  let bSum = 0; rhoP = 1;
  for (const k of bKeys) { bSum += sumVsop(c[k], rho) * rhoP; rhoP *= rho; }
  let rSum = 0; rhoP = 1;
  for (const k of rKeys) { rSum += sumVsop(c[k], rho) * rhoP; rhoP *= rho; }
  return {
    lon: mapTo0To360(RAD2DEG * (lSum / 1e8)),
    lat: mapToMinus90To90(RAD2DEG * (bSum / 1e8)),
    rad: rSum / 1e8
  };
}

function evalPlutoHelio(jd) {
  const T = (jd - J2000) / 36525.0;
  const J = 34.35 + 3034.9057 * T;
  const S = 50.08 + 1222.1138 * T;
  const P = 238.96 + 144.9600 * T;
  const c = COEFFS.pluto;
  let L = 0, B = 0, R = 0;
  for (let i = 0; i < c.g_PlutoArgumentCoefficients.length; i++) {
    const [cj, cs, cp] = c.g_PlutoArgumentCoefficients[i];
    const alpha = (cj * J + cs * S + cp * P) * DEG2RAD;
    const sa = Math.sin(alpha), ca = Math.cos(alpha);
    L += c.g_PlutoLongitudeCoefficients[i][0] * sa + c.g_PlutoLongitudeCoefficients[i][1] * ca;
    B += c.g_PlutoLatitudeCoefficients[i][0] * sa + c.g_PlutoLatitudeCoefficients[i][1] * ca;
    R += c.g_PlutoRadiusCoefficients[i][0] * sa + c.g_PlutoRadiusCoefficients[i][1] * ca;
  }
  return {
    lon: mapTo0To360(L / 1e6 + 238.958116 + 144.96 * T),
    lat: mapToMinus90To90(B / 1e6 - 3.908239),
    rad: R / 1e7 + 40.7241346
  };
}

const CHIRON_EPOCHS = [
  [2415020.5, 13.68200, 0.38050, 6.9240, 209.6800, 339.1200, 319.8500],
  [2433282.5, 13.64500, 0.37860, 6.9220, 209.6150, 339.4200,  33.1800],
  [2443448.5, 13.69480, 0.37860, 6.9230, 209.5910, 339.5050, 230.1250],
  [2450127.5, 13.69540, 0.38200, 6.9310, 209.3920, 339.5600,   0.0000],
  [2451545.0, 13.69619, 0.38209, 6.9310, 209.3906, 339.5613,  27.5620],
  [2459000.5, 13.70250, 0.37925, 6.9295, 209.3580, 339.9400, 172.3400],
  [2460800.5, 13.70420, 0.37911, 6.9292, 209.3550, 339.9620, 207.3300],
  [2469807.5, 13.69800, 0.38120, 6.9300, 209.3400, 339.8800,  22.3900]
];

function evalChironHelio(jd) {
  let best = CHIRON_EPOCHS[0];
  let minDist = Math.abs(jd - best[0]);
  for (let i = 1; i < CHIRON_EPOCHS.length; i++) {
    const d = Math.abs(jd - CHIRON_EPOCHS[i][0]);
    if (d < minDist) { minDist = d; best = CHIRON_EPOCHS[i]; }
  }
  const [jd0, a, e, iDeg, nodeDeg, argpDeg, m0Deg] = best;
  const nDeg = 0.9856076686 / (a * Math.sqrt(a));
  const mRad = mapTo0To360(m0Deg + nDeg * (jd - jd0)) * DEG2RAD;
  let E = mRad + e * Math.sin(mRad) * (1 + e * Math.cos(mRad));
  for (let k = 0; k < 8; k++) {
    const dE = (E - e * Math.sin(E) - mRad) / (1 - e * Math.cos(E));
    E -= dE;
    if (Math.abs(dE) < 1e-12) break;
  }
  const xv = a * (Math.cos(E) - e);
  const yv = a * Math.sqrt(1 - e * e) * Math.sin(E);
  const vRad = Math.atan2(yv, xv);
  const rAu = Math.sqrt(xv * xv + yv * yv);
  const uRad = argpDeg * DEG2RAD + vRad;
  const iRad = iDeg * DEG2RAD;
  const nodeRad = nodeDeg * DEG2RAD;
  const lonJ2000 = nodeRad + Math.atan2(Math.sin(uRad) * Math.cos(iRad), Math.cos(uRad));
  const latJ2000 = Math.asin(Math.sin(uRad) * Math.sin(iRad));
  const tCent = (jd - J2000) / 36525.0;
  const precessDeg = 1.396971 * tCent + 0.0003086 * tCent * tCent;
  return {
    lon: mapTo0To360(lonJ2000 * RAD2DEG + precessDeg),
    lat: mapToMinus90To90(latJ2000 * RAD2DEG),
    rad: rAu
  };
}

function getEllipticalDetails(jd, helioFn) {
  let jd0 = jd;
  const l0 = earthLon(jd0) * DEG2RAD;
  const b0 = earthLat(jd0) * DEG2RAD;
  const r0 = earthRad(jd0);
  const cosB0 = Math.cos(b0);

  let L = 0, B = 0, R = 0;
  let lPrev = 0, bPrev = 0, rPrev = 0, first = true, recalc = true;
  while (recalc) {
    const h = helioFn(jd0);
    L = h.lon; B = h.lat; R = h.rad;
    if (!first) {
      recalc = Math.abs(L - lPrev) > 1e-5 || Math.abs(B - bPrev) > 1e-5 || Math.abs(R - rPrev) > 1e-6;
    }
    first = false;
    lPrev = L; bPrev = B; rPrev = R;
    if (recalc) {
      const lr = L * DEG2RAD, br = B * DEG2RAD;
      const x = R * Math.cos(br) * Math.cos(lr) - r0 * cosB0 * Math.cos(l0);
      const y = R * Math.cos(br) * Math.sin(lr) - r0 * cosB0 * Math.sin(l0);
      const z = R * Math.sin(br) - r0 * Math.sin(b0);
      const dist = Math.sqrt(x*x + y*y + z*z);
      jd0 = jd - dist * 0.0057755183;
    }
  }
  const lr = L * DEG2RAD, br = B * DEG2RAD;
  const x = R * Math.cos(br) * Math.cos(lr) - r0 * cosB0 * Math.cos(l0);
  const y = R * Math.cos(br) * Math.sin(lr) - r0 * cosB0 * Math.sin(l0);
  const z = R * Math.sin(br) - r0 * Math.sin(b0);
  const dist = Math.sqrt(x*x + y*y + z*z);
  let lon = mapTo0To360(RAD2DEG * Math.atan2(y, x));
  let lat = RAD2DEG * Math.atan2(z, Math.sqrt(x*x + y*y));
  const ab = eclipticAberration(jd, lon, lat);
  lon += ab.X; lat += ab.Y;
  lon += fk5Long(jd, lon, lat);
  lat += fk5Lat(jd, lon);
  const nut = getNutationInLongitude(jd);
  lon = mapTo0To360(lon + getDecimal(0, 0, Math.abs(nut), nut >= 0));
  const equ = eclipticToEquatorial(lon, lat, getTrueObliquity(jd));
  return { lon, lat, dist, equ };
}

function moonArgs(jd) {
  const T = (jd - J2000) / 36525.0;
  const T2 = T*T, T3 = T2*T, T4 = T3*T;
  const Ldash = mapTo0To360(218.3164477 + 481267.88123421*T - 0.0015786*T2 + T3/538841 - T4/65194000);
  const D = mapTo0To360(297.8501921 + 445267.1114034*T - 0.0018819*T2 + T3/545868 - T4/113065000);
  const M = mapTo0To360(357.5291092 + 35999.0502909*T - 0.0001536*T2 + T3/24490000);
  const Mdash = mapTo0To360(134.9633964 + 477198.8675055*T + 0.0087414*T2 + T3/69699 - T4/14712000);
  const F = mapTo0To360(93.2720950 + 483202.0175233*T - 0.0036539*T2 - T3/3526000 + T4/863310000);
  const E = 1 - 0.002516*T - 0.0000074*T2;
  return { T, T2, T3, T4, Ldash, D, M, Mdash, F, E };
}

function moonCoords(jd) {
  const { T, Ldash, D, M, Mdash, F, E } = moonArgs(jd);
  const lr = Ldash*DEG2RAD, dr = D*DEG2RAD, mr = M*DEG2RAD, mdr = Mdash*DEG2RAD, fr = F*DEG2RAD;
  const E2 = E*E;
  const A1 = mapTo0To360(119.75 + 131.849*T)*DEG2RAD;
  const A2 = mapTo0To360(53.09 + 479264.290*T)*DEG2RAD;
  const A3 = mapTo0To360(313.45 + 481266.484*T)*DEG2RAD;
  const mc = COEFFS.moon;
  let sigL = 0, sigR = 0;
  for (let i = 0; i < mc.gMoonCoefficients1.length; i++) {
    const [cd, cm, cmd, cf] = mc.gMoonCoefficients1[i];
    const [ca, cb] = mc.gMoonCoefficients2[i];
    const arg = cd*dr + cm*mr + cmd*mdr + cf*fr;
    let mult = 1;
    if (cm === 1 || cm === -1) mult = E;
    else if (cm === 2 || cm === -2) mult = E2;
    sigL += ca * Math.sin(arg) * mult;
    sigR += cb * Math.cos(arg) * mult;
  }
  sigL += 3958*Math.sin(A1) + 1962*Math.sin(lr - fr) + 318*Math.sin(A2);
  let sigB = 0;
  for (let i = 0; i < mc.gMoonCoefficients3.length; i++) {
    const [cd, cm, cmd, cf] = mc.gMoonCoefficients3[i];
    const cb = mc.gMoonCoefficients4[i];
    const arg = cd*dr + cm*mr + cmd*mdr + cf*fr;
    let mult = 1;
    if (cm === 1 || cm === -1) mult = E;
    else if (cm === 2 || cm === -2) mult = E2;
    sigB += cb * Math.sin(arg) * mult;
  }
  sigB += -2235*Math.sin(lr) + 382*Math.sin(A3) + 175*Math.sin(A1 - fr) + 175*Math.sin(A1 + fr) + 127*Math.sin(lr - mdr) - 115*Math.sin(lr + mdr);
  const lon = mapTo0To360(Ldash + sigL/1e6 + getNutationInLongitude(jd)/3600.0);
  const lat = sigB / 1e6;
  const distKm = 385000.56 + sigR / 1000.0;
  return { lon, lat, distKm };
}

function meanNorthNode(jd) {
  const { T, T2, T3, T4 } = moonArgs(jd);
  return mapTo0To360(125.0445479 - 1934.1362891*T + 0.0020754*T2 + T3/467441 - T4/60616000);
}

function trueNorthNode(jd) {
  const { D, M, Mdash, F } = moonArgs(jd);
  let node = meanNorthNode(jd);
  const dr = D*DEG2RAD, mr = M*DEG2RAD, mdr = Mdash*DEG2RAD, fr = F*DEG2RAD;
  node -= 1.4979 * Math.sin(2*(dr - fr));
  node -= 0.1500 * Math.sin(mr);
  node -= 0.1226 * Math.sin(2*dr);
  node += 0.1176 * Math.sin(2*fr);
  node -= 0.0801 * Math.sin(2*(mdr - fr));
  return mapTo0To360(node);
}

function meanLilith(jd) {
  const { T, T2, T3, T4 } = moonArgs(jd);
  const perigee = mapTo0To360(83.3532465 + 4069.0137287*T - 0.0103200*T2 - T3/80053 + T4/18999000);
  return mapTo0To360(perigee + 180.0);
}

const ZODIAC_SIGNS = [
  ['Áries','♈','Fogo'],['Touro','♉','Terra'],['Gêmeos','♊','Ar'],['Câncer','♋','Água'],
  ['Leão','♌','Fogo'],['Virgem','♍','Terra'],['Libra','♎','Ar'],['Escorpião','♏','Água'],
  ['Sagitário','♐','Fogo'],['Capricórnio','♑','Terra'],['Aquário','♒','Ar'],['Peixes','♓','Água']
];

function diffAngle(a, b) {
  let d = a - b;
  if (d > 180) d -= 360;
  else if (d < -180) d += 360;
  return d;
}

function buildBody(id, name, symbol, category, longitude, latitude, rightAscension, declination, distance, distanceUnit, dailySpeed) {
  const lon = mapTo0To360(longitude);
  const signIndex = Math.min(11, Math.floor(lon / 30));
  const [signName, signSymbol, element] = ZODIAC_SIGNS[signIndex];
  const dms = getSexagesimal(lon - signIndex * 30);
  const isRetrograde = dailySpeed < 0;
  const formattedPosition = `${String(dms.radix).padStart(2,'0')}° ${signSymbol} ${String(dms.minutes).padStart(2,'0')}' ${dms.seconds.toFixed(1).padStart(4,'0')}"${isRetrograde ? ' Rx' : ''}`;
  return {
    id, name, symbol, category, longitude: lon, latitude, rightAscension, declination,
    distance, distanceUnit, dailySpeed, isRetrograde, signIndex, signName, signSymbol, element,
    degreeInSign: dms.radix, minutes: dms.minutes, seconds: Number(dms.seconds.toFixed(2)), formattedPosition
  };
}

function computeAll(unixMs, lngObserver) {
  const t0 = process.hrtime.bigint();
  const jd = unixMs / DAYMS - 0.5 + J1970;
  const dt = 0.005;
  const eps = getTrueObliquity(jd);
  const bodies = [];

  // Sol
  const sLon = sunApparentLon(jd), sLat = sunApparentLat(jd);
  const sEqu = eclipticToEquatorial(sLon, sLat, eps);
  const sSpeed = diffAngle(sunApparentLon(jd + dt), sunApparentLon(jd - dt)) / (2 * dt);
  bodies.push(buildBody('sun', 'Sol', '☉', 'Luminar', sLon, sLat, sEqu.rightAscension, sEqu.declination, earthRad(jd), 'UA', sSpeed));

  // Lua
  const mCur = moonCoords(jd), mPrev = moonCoords(jd - dt), mNext = moonCoords(jd + dt);
  const mEqu = eclipticToEquatorial(mCur.lon, mCur.lat, eps);
  bodies.push(buildBody('moon', 'Lua', '☽', 'Luminar', mCur.lon, mCur.lat, mEqu.rightAscension, mEqu.declination, mCur.distKm, 'km', diffAngle(mNext.lon, mPrev.lon)/(2*dt)));

  const specs = [
    ['mercury','Mercúrio','☿','Pessoal', (j) => evalVsopPlanet('mercury',['g_L0MercuryCoefficients','g_L1MercuryCoefficients','g_L2MercuryCoefficients','g_L3MercuryCoefficients','g_L4MercuryCoefficients','g_L5MercuryCoefficients'],['g_B0MercuryCoefficients','g_B1MercuryCoefficients','g_B2MercuryCoefficients','g_B3MercuryCoefficients','g_B4MercuryCoefficients'],['g_R0MercuryCoefficients','g_R1MercuryCoefficients','g_R2MercuryCoefficients','g_R3MercuryCoefficients'], j)],
    ['venus','Vênus','♀','Pessoal', (j) => evalVsopPlanet('venus',['g_L0VenusCoefficients','g_L1VenusCoefficients','g_L2VenusCoefficients','g_L3VenusCoefficients','g_L4VenusCoefficients','g_L5VenusCoefficients'],['g_B0VenusCoefficients','g_B1VenusCoefficients','g_B2VenusCoefficients','g_B3VenusCoefficients','g_B4VenusCoefficients'],['g_R0VenusCoefficients','g_R1VenusCoefficients','g_R2VenusCoefficients','g_R3VenusCoefficients'], j)],
    ['mars','Marte','♂','Pessoal', (j) => evalVsopPlanet('mars',['g_L0MarsCoefficients','g_L1MarsCoefficients','g_L2MarsCoefficients','g_L3MarsCoefficients','g_L4MarsCoefficients','g_L5MarsCoefficients'],['g_B0MarsCoefficients','g_B1MarsCoefficients','g_B2MarsCoefficients','g_B3MarsCoefficients','g_B4MarsCoefficients'],['g_R0MarsCoefficients','g_R1MarsCoefficients','g_R2MarsCoefficients','g_R3MarsCoefficients','g_R4MarsCoefficients'], j)],
    ['jupiter','Júpiter','♃','Social', (j) => evalVsopPlanet('jupiter',['g_L0JupiterCoefficients','g_L1JupiterCoefficients','g_L2JupiterCoefficients','g_L3JupiterCoefficients','g_L4JupiterCoefficients','g_L5JupiterCoefficients'],['g_B0JupiterCoefficients','g_B1JupiterCoefficients','g_B2JupiterCoefficients','g_B3JupiterCoefficients','g_B4JupiterCoefficients','g_B5JupiterCoefficients'],['g_R0JupiterCoefficients','g_R1JupiterCoefficients','g_R2JupiterCoefficients','g_R3JupiterCoefficients','g_R4JupiterCoefficients','g_R5JupiterCoefficients'], j)],
    ['saturn','Saturno','♄','Social', (j) => evalVsopPlanet('saturn',['g_L0SaturnCoefficients','g_L1SaturnCoefficients','g_L2SaturnCoefficients','g_L3SaturnCoefficients','g_L4SaturnCoefficients','g_L5SaturnCoefficients'],['g_B0SaturnCoefficients','g_B1SaturnCoefficients','g_B2SaturnCoefficients','g_B3SaturnCoefficients','g_B4SaturnCoefficients','g_B5SaturnCoefficients'],['g_R0SaturnCoefficients','g_R1SaturnCoefficients','g_R2SaturnCoefficients','g_R3SaturnCoefficients','g_R4SaturnCoefficients','g_R5SaturnCoefficients'], j)],
    ['chiron','Quíron','⚷','Centauro', evalChironHelio],
    ['uranus','Urano','♅','Geracional', (j) => evalVsopPlanet('uranus',['g_L0UranusCoefficients','g_L1UranusCoefficients','g_L2UranusCoefficients','g_L3UranusCoefficients','g_L4UranusCoefficients'],['g_B0UranusCoefficients','g_B1UranusCoefficients','g_B2UranusCoefficients','g_B3UranusCoefficients','g_B4UranusCoefficients'],['g_R0UranusCoefficients','g_R1UranusCoefficients','g_R2UranusCoefficients','g_R3UranusCoefficients','g_R4UranusCoefficients'], j)],
    ['neptune','Netuno','♆','Geracional', (j) => evalVsopPlanet('neptune',['g_L0NeptuneCoefficients','g_L1NeptuneCoefficients','g_L2NeptuneCoefficients','g_L3NeptuneCoefficients','g_L4NeptuneCoefficients'],['g_B0NeptuneCoefficients','g_B1NeptuneCoefficients','g_B2NeptuneCoefficients','g_B3NeptuneCoefficients','g_B4NeptuneCoefficients'],['g_R0NeptuneCoefficients','g_R1NeptuneCoefficients','g_R2NeptuneCoefficients','g_R3NeptuneCoefficients'], j)],
    ['pluto','Plutão','♇','Geracional', evalPlutoHelio]
  ];

  for (const [id, name, symbol, cat, fn] of specs) {
    const cur = getEllipticalDetails(jd, fn);
    const prev = getEllipticalDetails(jd - dt, fn);
    const next = getEllipticalDetails(jd + dt, fn);
    const speed = diffAngle(next.lon, prev.lon) / (2 * dt);
    bodies.push(buildBody(id, name, symbol, cat, cur.lon, cur.lat, cur.equ.rightAscension, cur.equ.declination, cur.dist, 'UA', speed));
  }

  // Nodos Lunares (Médio e Verdadeiro) e Lilith
  const mnn = meanNorthNode(jd);
  const mnnSpeed = diffAngle(meanNorthNode(jd + dt), meanNorthNode(jd - dt)) / (2 * dt);
  const mnnEqu = eclipticToEquatorial(mnn, 0, eps);
  bodies.push(buildBody('mean_north_node', 'Nodo Norte (Médio)', '☊', 'Ponto Lunar', mnn, 0, mnnEqu.rightAscension, mnnEqu.declination, 0, '-', mnnSpeed));

  const msn = mapTo0To360(mnn + 180);
  const msnEqu = eclipticToEquatorial(msn, 0, eps);
  bodies.push(buildBody('mean_south_node', 'Nodo Sul (Médio)', '☋', 'Ponto Lunar', msn, 0, msnEqu.rightAscension, msnEqu.declination, 0, '-', mnnSpeed));

  const nn = trueNorthNode(jd);
  const nnSpeed = diffAngle(trueNorthNode(jd + dt), trueNorthNode(jd - dt)) / (2 * dt);
  const nnEqu = eclipticToEquatorial(nn, 0, eps);
  bodies.push(buildBody('true_north_node', 'Nodo Norte (Verdadeiro)', '☊', 'Ponto Lunar', nn, 0, nnEqu.rightAscension, nnEqu.declination, 0, '-', nnSpeed));

  const sn = mapTo0To360(nn + 180);
  const snEqu = eclipticToEquatorial(sn, 0, eps);
  bodies.push(buildBody('true_south_node', 'Nodo Sul (Verdadeiro)', '☋', 'Ponto Lunar', sn, 0, snEqu.rightAscension, snEqu.declination, 0, '-', nnSpeed));

  const lil = meanLilith(jd);
  const lilSpeed = diffAngle(meanLilith(jd + dt), meanLilith(jd - dt)) / (2 * dt);
  const lilEqu = eclipticToEquatorial(lil, 0, eps);
  bodies.push(buildBody('mean_lilith', 'Lilith (Lua Negra Média)', '⚸', 'Ponto Lunar', lil, 0, lilEqu.rightAscension, lilEqu.declination, 0, '-', lilSpeed));

  const aspectDefs = [
    ['Conjunção', '☌', 0, 8, '#f59e0b'],
    ['Sextil', '⚹', 60, 5, '#10b981'],
    ['Quadratura', '□', 90, 7, '#ef4444'],
    ['Trígono', '△', 120, 7, '#3b82f6'],
    ['Oposição', '☍', 180, 8, '#ec4899']
  ];
  const aspects = [];
  const limit = Math.min(11, bodies.length);
  for (let i = 0; i < limit; i++) {
    for (let j = i + 1; j < limit; j++) {
      let diff = Math.abs(bodies[i].longitude - bodies[j].longitude);
      if (diff > 180) diff = 360 - diff;
      for (const [aname, asym, exact, maxOrb, color] of aspectDefs) {
        const orb = Math.abs(diff - exact);
        if (orb <= maxOrb) {
          aspects.push({
            body1: bodies[i].name, symbol1: bodies[i].symbol,
            body2: bodies[j].name, symbol2: bodies[j].symbol,
            aspectName: aname, aspectSymbol: asym,
            angle: Number(diff.toFixed(2)), orb: Number(orb.toFixed(2)), color
          });
        }
      }
    }
  }

  const T = (jd - J2000) / 36525.0;
  const gmst = mapTo0To360(280.46061837 + 360.98564736629 * (jd - J2000) + 0.000387933 * T * T - (T**3)/38710000.0);
  const lst = ((gmst + Number(lngObserver)) * DEG2H % 24 + 24) % 24;
  const elapsedUs = Math.round(Number(process.hrtime.bigint() - t0) / 1000);

  return {
    engine: 'Motor VSOP87 + ELP2000 (Espelho JS do Código Rust — Compile rust-astro-engine com cargo para binário nativo)',
    julianDay: jd,
    unixMs,
    computationTimeMicros: elapsedUs,
    astronomy: {
      trueObliquity: eps,
      meanObliquity: getMeanObliquity(jd),
      nutationLongitudeArcsec: getNutationInLongitude(jd),
      nutationObliquityArcsec: getNutationInObliquity(jd),
      localSiderealTimeHours: lst
    },
    bodies,
    aspects
  };
}

module.exports = { computeAll };
