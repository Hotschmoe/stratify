# Stratify Equations Reference

> **Auto-generated from source code. Do not edit manually.**
>
> Regenerate with: `cargo run --bin gen-equations`

This document lists all mathematical formulas used in Stratify calculations.
Each equation includes its formula, code reference, source location, and assumptions.
Engineers can use this as a single reference to audit the underlying mathematics.

## Sign Conventions

| Quantity | Positive Direction |
|----------|-------------------|
| Loads | Downward (gravity direction) |
| Moment | Tension on bottom fiber (sagging) |
| Shear | Left side up relative to right |
| Deflection | Downward |
| Reactions | Upward (resisting gravity) |

---

## Section Properties

### Rectangular Area

Cross-sectional area of rectangular section

**Formula:** `A = b * d`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| A | Cross-sectional area | in^2 |
| b | Width | in |
| d | Depth | in |

**Reference:** Fundamental Mechanics

**Source:** [`rectangular_area`](equations/section.rs)

**Assumptions:**
- Solid rectangular section

---

### Rectangular Section Modulus

Elastic section modulus of rectangular section

**Formula:** `S = bd^2/6`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| S | Section modulus | in^3 |
| b | Width | in |
| d | Depth | in |

**Reference:** Fundamental Mechanics

**Source:** [`rectangular_section_modulus`](equations/section.rs)

**Assumptions:**
- Solid rectangular section
- Bending about strong axis

---

### Rectangular Moment of Inertia

Moment of inertia of rectangular section about centroidal axis

**Formula:** `I = bd^3/12`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| I | Moment of inertia | in^4 |
| b | Width | in |
| d | Depth | in |

**Reference:** Fundamental Mechanics

**Source:** [`rectangular_moment_of_inertia`](equations/section.rs)

**Assumptions:**
- Solid rectangular section
- About centroidal axis

---

## Reactions

### Point Load Reactions

Support reactions for concentrated load at distance a from left support

**Formula:** `R1 = P(L-a)/L, R2 = Pa/L`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| P | Point load magnitude | lb |
| a | Distance from left support to load | ft |
| L | Span length | ft |
| R_1 | Left reaction | lb |
| R_2 | Right reaction | lb |

**Reference:** Roark's 8ed, Table 8.1, Case 1a

**Source:** [`point_load_reactions`](equations/beam.rs)

**Assumptions:**
- Simply-supported (pin-roller)
- Load is perpendicular to beam axis

---

### Uniform Load Reactions

Support reactions for uniformly distributed load over full span

**Formula:** `R1 = R2 = wL/2`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| w | Uniform load intensity | plf |
| L | Span length | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 2a

**Source:** [`uniform_load_reactions`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Symmetric loading

---

### Partial Uniform Load Reactions

Reactions for uniform load from position a to b

**Formula:** `R1 = W(L-c)/L, R2 = Wc/L where W = w(b-a), c = (a+b)/2`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| W | Total load | lb |
| c | Centroid position | ft |
| a | Load start position | ft |
| b | Load end position | ft |

**Reference:** Fundamental Mechanics

**Source:** [`partial_uniform_reactions`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Load treated as resultant at centroid for reactions

---

### Cantilever Uniform Load Reactions

Reaction and fixed-end moment for cantilever with uniform load

**Formula:** `R = wL, M = wL^2/2`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| R | Support reaction | lb |
| M | Fixed-end moment | ft-lb |

**Reference:** Roark's 8ed, Table 8.1, Case 2b

**Source:** [`cantilever_uniform_reactions`](equations/beam.rs)

**Assumptions:**
- Fixed at one end, free at other

---

### Cantilever Point Load Reactions

Reaction and fixed-end moment for cantilever with point load

**Formula:** `R = P, M = Pa`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| R | Support reaction | lb |
| M | Fixed-end moment | ft-lb |
| a | Distance from support to load | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 1b

**Source:** [`cantilever_point_reactions`](equations/beam.rs)

**Assumptions:**
- Fixed at one end, free at other

---

### Propped Cantilever Reactions

Reactions for beam fixed at left, pinned at right, with uniform load

**Formula:** `R_A = 5wL/8, R_B = 3wL/8, M_A = wL^2/8`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| R_A | Reaction at fixed end | lb |
| R_B | Reaction at pinned end | lb |
| M_A | Moment at fixed end | ft-lb |

**Reference:** Roark's 8ed, Table 8.1, Case 2c

**Source:** [`fixed_pinned_uniform_reactions`](equations/beam.rs)

**Assumptions:**
- Fixed-pinned supports
- Asymmetric reactions

---

## Internal Forces

### Point Load Shear

Shear force at position x for concentrated load

**Formula:** `V(x) = R1 for x < a, V(x) = R1 - P for x >= a`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| V | Shear force | lb |
| x | Position along beam | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 1a

**Source:** [`point_load_shear`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Positive shear: left side up

---

### Point Load Moment

Bending moment at position x for concentrated load

**Formula:** `M(x) = R1*x for x <= a, M_max = Pa(L-a)/L`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| M | Bending moment | ft-lb |
| x | Position along beam | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 1a

**Source:** [`point_load_moment`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Positive moment: tension on bottom

---

### Uniform Load Shear

Shear force at position x for uniform load

**Formula:** `V(x) = w(L/2 - x)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| V | Shear force | lb |
| x | Position along beam | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 2a

**Source:** [`uniform_load_shear`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Linear variation

---

### Uniform Load Moment

Bending moment at position x for uniform load

**Formula:** `M(x) = wx(L-x)/2`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| M | Bending moment | ft-lb |
| x | Position along beam | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 2a

**Source:** [`uniform_load_moment`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Parabolic distribution

---

### Maximum Moment for Uniform Load

Maximum bending moment at midspan for uniform load

**Formula:** `M_max = wL^2/8`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| M_("max") | Maximum moment | ft-lb |
| w | Uniform load | plf |
| L | Span length | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 2a

**Source:** [`uniform_load_moment`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Occurs at midspan

---

### Partial Uniform Load Moment

Moment at position x for partial uniform load

**Formula:** `M(x) = R1*x - w(x-a)^2/2 for a < x < b`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| M | Bending moment | ft-lb |
| x | Position along beam | ft |

**Reference:** Fundamental Mechanics

**Source:** [`partial_uniform_moment`](equations/beam.rs)

**Assumptions:**
- Simply-supported
- Superposition of uniform load segment

---

### Partial Uniform Load Shear

Shear at position x for partial uniform load

**Formula:** `V(x) = R1 - w(x-a) for a < x < b`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| V | Shear force | lb |
| x | Position along beam | ft |

**Reference:** Fundamental Mechanics

**Source:** [`partial_uniform_shear`](equations/beam.rs)

**Assumptions:**
- Simply-supported

---

### Fixed-Fixed End Moments

End moments for beam fixed at both ends with uniform load

**Formula:** `M_A = M_B = wL^2/12 (hogging)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| M_A | Moment at left support | ft-lb |
| M_B | Moment at right support | ft-lb |

**Reference:** Roark's 8ed, Table 8.1, Case 2e

**Source:** [`fixed_fixed_uniform_end_moments`](equations/beam.rs)

**Assumptions:**
- Both ends fully fixed
- Symmetric loading

---

### Fixed-Fixed Max Positive Moment

Maximum positive moment at midspan for fixed-fixed beam

**Formula:** `M_max = wL^2/24 (sagging at midspan)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| M_("max") | Maximum positive moment | ft-lb |

**Reference:** Roark's 8ed, Table 8.1, Case 2e

**Source:** [`fixed_fixed_uniform_max_positive_moment`](equations/beam.rs)

**Assumptions:**
- Both ends fully fixed
- Occurs at midspan

---

### Propped Cantilever Max Positive Moment

Maximum positive moment for propped cantilever with uniform load

**Formula:** `M_max = 9wL^2/128 at x = 3L/8`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| M_("max") | Maximum positive moment | ft-lb |

**Reference:** Roark's 8ed, Table 8.1, Case 2c

**Source:** [`fixed_pinned_uniform_max_positive_moment`](equations/beam.rs)

**Assumptions:**
- Fixed-pinned supports
- Occurs at 3L/8 from fixed end

---

## Stresses

### Bending Stress

Maximum bending stress at extreme fiber

**Formula:** `f_b = M / S`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| f_b | Bending stress | psi |
| M | Bending moment | in-lb |
| S | Section modulus | in^3 |

**Reference:** Fundamental Mechanics

**Source:** [`calculate`](calculations/beam.rs)

**Assumptions:**
- Linear elastic material
- Plane sections remain plane

---

### Shear Stress (Rectangular)

Maximum shear stress in rectangular section

**Formula:** `f_v = 3V / (2bd)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| f_v | Shear stress | psi |
| V | Shear force | lb |
| b | Width | in |
| d | Depth | in |

**Reference:** NDS 2018 Section 3.4.2

**Source:** [`calculate`](calculations/beam.rs)

**Assumptions:**
- Rectangular section
- Parabolic shear distribution
- Max at neutral axis

---

## Deflections

### Point Load Deflection

Deflection at position x for concentrated load

**Formula:** `delta(x) = Pbx(L^2 - b^2 - x^2) / (6EIL) for x <= a`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| delta | Deflection | in |
| E | Modulus of elasticity | psi |
| I | Moment of inertia | in^4 |
| b | L - a | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 1a

**Source:** [`point_load_deflection`](equations/beam.rs)

**Assumptions:**
- Linear elastic material
- Small deflections

---

### Uniform Load Deflection

Deflection at position x for uniform load

**Formula:** `delta(x) = wx(L^3 - 2Lx^2 + x^3) / (24EI)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| delta | Deflection | in |
| E | Modulus of elasticity | psi |
| I | Moment of inertia | in^4 |

**Reference:** Roark's 8ed, Table 8.1, Case 2a

**Source:** [`uniform_load_deflection`](equations/beam.rs)

**Assumptions:**
- Linear elastic material
- Small deflections

---

### Maximum Deflection for Uniform Load

Maximum deflection at midspan for uniform load

**Formula:** `delta_max = 5wL^4 / (384EI)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| delta_("max") | Maximum deflection | in |
| w | Uniform load | lb/in |
| L | Span length | in |
| E | Modulus of elasticity | psi |
| I | Moment of inertia | in^4 |

**Reference:** Roark's 8ed, Table 8.1, Case 2a

**Source:** [`uniform_load_deflection`](equations/beam.rs)

**Assumptions:**
- Linear elastic
- Occurs at midspan
- Small deflections

---

### Fixed-Fixed Max Deflection

Maximum deflection at midspan for fixed-fixed beam with uniform load

**Formula:** `delta_max = wL^4 / (384EI)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| delta_("max") | Maximum deflection | in |

**Reference:** Roark's 8ed, Table 8.1, Case 2e

**Source:** [`fixed_fixed_uniform_max_deflection`](equations/beam.rs)

**Assumptions:**
- Both ends fully fixed
- 1/5 of simply-supported deflection

---

### Cantilever Uniform Load Max Deflection

Maximum deflection at free end for cantilever with uniform load

**Formula:** `delta_max = wL^4 / (8EI)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| delta_("max") | Maximum deflection at free end | in |

**Reference:** Roark's 8ed, Table 8.1, Case 2b

**Source:** [`cantilever_uniform_max_deflection`](equations/beam.rs)

**Assumptions:**
- Fixed at one end
- Deflection at free end

---

## Fixed-End Moments

### FEM for Uniform Load

Fixed-end moments for uniform load over entire span

**Formula:** `FEM_A = -wL^2/12, FEM_B = +wL^2/12`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| FEM_A | Fixed-end moment at A | ft-lb |
| FEM_B | Fixed-end moment at B | ft-lb |

**Reference:** Roark's 8ed, Table 8.1, Case 2e

**Source:** [`fem_uniform_full`](equations/beam.rs)

**Assumptions:**
- Fully fixed ends
- Used in moment distribution method

---

### FEM for Point Load

Fixed-end moments for point load at distance a

**Formula:** `FEM_A = -Pab^2/L^2, FEM_B = +Pa^2b/L^2`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| FEM_A | Fixed-end moment at A | ft-lb |
| FEM_B | Fixed-end moment at B | ft-lb |
| b | L - a | ft |

**Reference:** Roark's 8ed, Table 8.1, Case 1e

**Source:** [`fem_point_load`](equations/beam.rs)

**Assumptions:**
- Fully fixed ends
- Used in moment distribution method

---

### FEM for Partial Uniform Load

Fixed-end moments for partial uniform load (numerical integration)

**Formula:** `FEM = sum(P_i * FEM_i) (discrete approximation)`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| FEM | Fixed-end moment | ft-lb |

**Reference:** Fundamental Mechanics

**Source:** [`fem_partial_uniform`](equations/beam.rs)

**Assumptions:**
- Numerical integration of point load FEMs
- 20 segments

---

## Adjustment Factors

### NDS Adjusted Bending Strength

Reference bending design value multiplied by all adjustment factors

**Formula:** `F'_b = F_b * C_D * C_M * C_t * C_L * C_F * C_fu * C_i * C_r`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| F'_b | Adjusted bending design value | psi |
| F_b | Reference bending design value | psi |
| C_D | Load duration factor | - |
| C_M | Wet service factor | - |
| C_t | Temperature factor | - |
| C_L | Beam stability factor | - |
| C_F | Size factor | - |
| C_("fu") | Flat use factor | - |
| C_i | Incising factor | - |
| C_r | Repetitive member factor | - |

**Reference:** NDS 2018 Section 4.3

**Source:** [`apply_bending_factors`](nds_factors.rs)

**Assumptions:**
- Sawn lumber per NDS 2018
- ASD method

---

### NDS Adjusted Shear Strength

Reference shear design value multiplied by applicable adjustment factors

**Formula:** `F'_v = F_v * C_D * C_M * C_t * C_i`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| F'_v | Adjusted shear design value | psi |
| F_v | Reference shear design value | psi |

**Reference:** NDS 2018 Section 4.3

**Source:** [`apply_shear_factors`](nds_factors.rs)

**Assumptions:**
- Sawn lumber per NDS 2018
- ASD method

---

### NDS Adjusted Modulus of Elasticity

Reference modulus adjusted for service conditions

**Formula:** `E' = E * C_M * C_t * C_i`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| E' | Adjusted modulus of elasticity | psi |
| E | Reference modulus of elasticity | psi |

**Reference:** NDS 2018 Section 4.3

**Source:** [`apply_modulus_factors`](nds_factors.rs)

**Assumptions:**
- Sawn lumber per NDS 2018

---

## Design Checks

### NDS Bending Unity Ratio

Demand/capacity ratio for bending stress check

**Formula:** `f_b / F'_b <= 1.0`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| f_b | Actual bending stress | psi |
| F'_b | Adjusted allowable bending stress | psi |

**Reference:** NDS 2018 Section 3.3

**Source:** [`calculate`](calculations/beam.rs)

**Assumptions:**
- Unity ratio <= 1.0 indicates adequate capacity

---

### NDS Shear Unity Ratio

Demand/capacity ratio for shear stress check

**Formula:** `f_v / F'_v <= 1.0`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| f_v | Actual shear stress | psi |
| F'_v | Adjusted allowable shear stress | psi |

**Reference:** NDS 2018 Section 3.4

**Source:** [`calculate`](calculations/beam.rs)

**Assumptions:**
- Unity ratio <= 1.0 indicates adequate capacity

---

### Deflection Limit

Serviceability check for maximum deflection

**Formula:** `delta <= L/n where n = 240, 360, etc.`

**Variables:**

| Symbol | Description | Units |
|--------|-------------|-------|
| delta | Actual deflection | in |
| L | Span length | in |
| n | Deflection ratio (240, 360, etc.) | - |

**Reference:** ASCE 7-22 Section Table 1604.3

**Source:** [`calculate`](calculations/beam.rs)

**Assumptions:**
- IBC Table 1604.3 limits
- Floor/roof specific limits

---

## Statistics

- **Total Equations:** 35
- **Categories:** 8

## How to Audit

1. Find the equation you want to verify in the sections above
2. Check the **Reference** for the original source (Roark's, NDS, ASCE 7, etc.)
3. Click the **Source** link to view the implementation code
4. Run `cargo test` to verify equations against known values

For questions or issues, see the main README.md.
