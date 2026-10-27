# Samples tour

The **Samples** browser (File > Samples..., or the Resources tab) lists 126 worksheets, with
groups, a search box and a preview. Opening a sample puts it in a new tab, where you can change it
freely: nothing you do changes the original. A sample that has input fields carries an **input form**
tag. The Samples browser never runs a sample's Python; its preview shows the source, and opening the
sample asks first.

## Sheets

Written in the sheet format. They are listed first.

| Group | What is in it |
| --- | --- |
| Sheet syntax (11) | Beam bending, a short column, a floor-load table with vectors and matrices, a moment diagram with a plot, LaTeX prose, mixing with raw Calcpad and loops, beam and cantilever deflection, a three-span beam explorer with Python and Plotly, a beam deflection derived with the Symbolic dialog, polynomial roots. |
| 3D plots (5) | A plate deflection surface, a column interaction surface, a space frame in 3D lines, a mode-shape explorer with a dropdown, a 3D bar chart of footing reactions. |
| Plots (2) | A plot gallery of implicit curves, functions and polylines on a grid, and an animated beam with a moving load (Play, Pause, a slider). |
| Physics (1) | A tennis ball in flight: drag, spin and the sideways force from spin, integrated in a Python block. |

## Calcpad worksheets

The bundled worksheets are the CalcpadCE examples, with their input fields written as `#UI`, plus a
few of our own.

| Group | What is in it |
| --- | --- |
| Beams (26) | Cantilevers, propped cantilevers, simply supported and fully restrained beams under point, moment, uniform, linear and partial loads; deflection of a cantilever and of a simply supported beam; a continuous beam by the force method. |
| Reinforced Concrete (27) | Bars and areas, bending and shear of rectangular and tee sections, columns under axial force and one or two-way bending, an interaction diagram, serviceability, two-way slab panels with every edge condition, punching shear, detailing for ductility. |
| Steel (3) | Section properties of I-sections: by formula, from a catalogue with a drawing, from an Excel catalogue. |
| Loads (4) | Snow, wind and footing-pad optimisation worksheets. |
| Frames (3) | Plane frames with prismatic and tapered members, and a multi-storey frame. |
| Dynamics (4) | Free vibrations of a tapered pole, with plain, Plotly and SVG animations; drop-weight impact on a beam. |
| Nonlinear (2) | A prestressed truss with material and geometric nonlinearity, and a nonlinear truss with an animation. |
| Soil Mechanics (1) | Stress and deformation in soil. |
| Sections (1) | Properties of a built-up section. |
| Maths & plotting (31) | Getting started (calculating, functions and plots, unit conversion), matrices (creation, indexing, operators, structural and maths functions), interpolation, custom units (currency and information), numerical differentiation and integration, roots and systems of equations, Newton's method with animation, a Monte Carlo estimate, elastic solutions and finite-element worksheets for deep beams and slabs. |
| Python (5) | A beam moment diagram with matplotlib, a function plot with matplotlib, a beam explorer with Plotly, a continuous beam with Python results in raw JavaScript Plotly, and a plane truss by the stiffness method. |

Good places to start:

* **Getting Started: Calculating**, **Functions and Plots** and **Units and Conversion**, to see
  the Calcpad language at its simplest.
* **Beam bending with real units** (sheet) and **Bending moment diagram** (sheet), the shortest
  complete sheets.
* **Matrix** worksheets, for a tour of the vector and matrix functions.
* **Beam Explorer: Plotly from Python**, for the Python and Plotly route.

The samples that were written for other design practices apply rules specific to those practices.
Read a sample as an example of the technique and check its assumptions before reusing a number.
