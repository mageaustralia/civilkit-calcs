# Samples

These worksheets, and the images, included sheets and data files they use, are copied
from the CalcpadCE repository (https://github.com/imartincei/CalcpadCE, commit
0cf438f, folder `Examples/`), MIT licence, Copyright (c) 2026 CalcpadCE Contributors.
The copies keep upstream's folder layout under this folder.

The copies differ from upstream in one way: a deprecated stored input field
`name = ? {default}` (CalcpadCE's linter warns CPD-3419) is written as the `#UI`
declaration its migration guide gives, `#UI name = default`, keeping the name, the value
and the unit. tools/gen_samples.mjs does it (src/wasm/legacy-input.ts) and only keeps a
converted sheet when the engine renders it exactly like the original, apart from the
input widget markup (DEVIATIONS.md, D8). Worksheets:

- `Examples/Structural/Cantilevers/Concentrated Force.cpd`
- `Examples/Structural/Cantilevers/Concentrated Moment.cpd`
- `Examples/Structural/Cantilevers/Linearly Distributed Load.cpd`
- `Examples/Structural/Cantilevers/Partially Distributed Load.cpd`
- `Examples/Structural/Cantilevers/Uniformly Distributed Load.cpd`
- `Examples/Structural/Continuous Beam/Continuous Beam.cpd`
- `Examples/Structural/Deflections/Cantilever with Concentrated Load.cpd`
- `Examples/Structural/Deflections/Cantilever with Unifrom Load.cpd`
- `Examples/Structural/Deflections/Simply Supported Beam with Concentrated Load.cpd`
- `Examples/Structural/Deflections/Simply Supported Beam with Unifrom Load.cpd`
- `Examples/Structural/Frames/Frame-Var.cpd`
- `Examples/Structural/Frames/Frame.cpd`
- `Examples/Structural/Frames/Multi-Story Frame.cpd`
- `Examples/Structural/Fully Restrained Beams/Concentrated Force.cpd`
- `Examples/Structural/Fully Restrained Beams/Concentrated Moment.cpd`
- `Examples/Structural/Fully Restrained Beams/Linearly Distributed Load.cpd`
- `Examples/Structural/Fully Restrained Beams/Partially Distributed Load.cpd`
- `Examples/Structural/Fully Restrained Beams/Uniformly Distributed Load.cpd`
- `Examples/Structural/Loads/Optimization of Foundation Pad Dimensions.cpd`
- `Examples/Structural/Loads/Snow Drift.cpd`
- `Examples/Structural/Loads/Snow Load.cpd`
- `Examples/Structural/Loads/Wind Load.cpd`
- `Examples/Structural/Nonlinear/Material and Geometric Nonlinear Analysis of Prestressed Asymmetric Biot Truss.cpd`
- `Examples/Structural/Nonlinear/Nonlinear Analysis of a Von-Mises Truss Animated.cpd`
- `Examples/Structural/Propped Cantilevers/Concentrated Force.cpd`
- `Examples/Structural/Propped Cantilevers/Concentrated Moment.cpd`
- `Examples/Structural/Propped Cantilevers/Linearly Distributed Load.cpd`
- `Examples/Structural/Propped Cantilevers/Partially Distributed Load.cpd`
- `Examples/Structural/Propped Cantilevers/Uniformly Distributed Load.cpd`
- `Examples/Structural/Reinforced Concrete/Areas of Reinforcement Bars.cpd`
- `Examples/Structural/Reinforced Concrete/Bending Capacity of Tee Section.cpd`
- `Examples/Structural/Reinforced Concrete/Bending Design of Rectangular Section.cpd`
- `Examples/Structural/Reinforced Concrete/Bending Design of Tee Section.cpd`
- `Examples/Structural/Reinforced Concrete/Column Design for Bending and Axial Force.cpd`
- `Examples/Structural/Reinforced Concrete/Column Design for Biaxial Bending and Axial Force (with Units).cpd`
- `Examples/Structural/Reinforced Concrete/Column Design for Biaxial Bending and Axial Force.cpd`
- `Examples/Structural/Reinforced Concrete/Combined Bending, Shear and Torsion.cpd`
- `Examples/Structural/Reinforced Concrete/Interaction Diagram of RC Section.cpd`
- `Examples/Structural/Reinforced Concrete/SLS Design of Beam with Rectangular Section.cpd`
- `Examples/Structural/Reinforced Concrete/SLS Design of Beam with Tee Section.cpd`
- `Examples/Structural/Reinforced Concrete/Shear Design of Rectangular Section.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Fully Restrained Slab Panel.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Restrained Adjacent Edges.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Restrained Both Longer Edges.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Restrained Both Shorter Edges.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Restrained Longer Edge.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Restrained Panel with Unrestrained Longer Edge.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Restrained Panel with Unrestrained Shorter Edge.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Restrained Shorter Edge.cpd`
- `Examples/Structural/Reinforced Concrete - Plates/Simply Supported Slab Panel.cpd`
- `Examples/Structural/Reinforced Concrete - Punching/Corner Column.cpd`
- `Examples/Structural/Reinforced Concrete - Punching/Edge Column.cpd`
- `Examples/Structural/Reinforced Concrete - Punching/Internal Column.cpd`
- `Examples/Structural/Reinforced Concrete - Seismic Detailing/Beam Detailing for DCM.cpd`
- `Examples/Structural/Reinforced Concrete - Seismic Detailing/Column Detailing for DCM.cpd`
- `Examples/Structural/Reinforced Concrete - Seismic Detailing/Shear Wall Detailing for DCM.cpd`
- `Examples/Structural/Simply Supported Beams/Concentrated Force.cpd`
- `Examples/Structural/Simply Supported Beams/Concentrated Forces.cpd`
- `Examples/Structural/Simply Supported Beams/Concentrated Moment.cpd`
- `Examples/Structural/Simply Supported Beams/Linearly Distributed Load.cpd`
- `Examples/Structural/Simply Supported Beams/Partially Distributed Load.cpd`
- `Examples/Structural/Simply Supported Beams/Uniformly Distributed Load.cpd`
- `Examples/Structural/Soil Mechanics/Soil Stress and Deformations.cpd`
- `Examples/Structural/Steel/I-Section Properties Excel.cpd`
- `Examples/Structural/Steel/I-Section Properties and Drawing.cpd`
- `Examples/Structural/Steel/I-Section Properties.cpd`
- `Examples/Structural/Structural Dynamics/Beam Impact Analysis Animated.cpd`
- `Examples/Structural/Structural Dynamics/Free Vibrations of Steel Pole Animated.cpd`
- `Examples/Structural/Structural Dynamics/Free Vibrations of Steel Pole Plotly Animated.cpd`
- `Examples/Structural/Structural Dynamics/Free Vibrations of Steel Pole.cpd`
- `Examples/Engineering/Getting Started/Calculating.cpd`
- `Examples/Engineering/Getting Started/Functions.cpd`
- `Examples/Engineering/Getting Started/Unit Conversion.cpd`
- `Examples/Engineering/Matrices/Matrix Creational Functions.cpd`
- `Examples/Engineering/Matrices/Matrix Data Functions.cpd`
- `Examples/Engineering/Matrices/Matrix Initialization and Indexing.cpd`
- `Examples/Engineering/Matrices/Matrix Math Functions.cpd`
- `Examples/Engineering/Matrices/Matrix Operators.cpd`
- `Examples/Engineering/Matrices/Matrix Structural Functions.cpd`
- `Examples/Engineering/Numerical/Monte Carlo Pi.cpd`
- `Examples/Engineering/Numerical/Newton's Method Animated.cpd`
- `Examples/Engineering/Numerical/Numerical Differentiation.cpd`
- `Examples/Engineering/Numerical/Numerical Integration.cpd`
- `Examples/Engineering/Numerical/Rayleigh Quotient Iteration.cpd`
- `Examples/Engineering/Numerical/System of Nonlinear Equations.cpd`
- `Examples/Engineering/Interpolation/Double Interpolation.cpd`
- `Examples/Engineering/Interpolation/Interpolation of Functions.cpd`
- `Examples/Engineering/Interpolation/Interpolation.cpd`
- `Examples/Engineering/Finite Elements/Deep Beam FEA.cpd`
- `Examples/Engineering/Finite Elements/Flat Slab FEA Optimized.cpd`
- `Examples/Engineering/Finite Elements/Flat Slab FEA.cpd`
- `Examples/Engineering/Finite Elements/Rectangular Slab FEA.cpd`
- `Examples/Engineering/Elastic Analysis/Deep Beam.cpd`
- `Examples/Engineering/Elastic Analysis/Rectangular Slab.cpd`
- `Examples/Engineering/Elastic Analysis/Saint-Venant Torsion.cpd`
- `Examples/Engineering/Custom Units/Currency Units.cpd`
- `Examples/Engineering/Custom Units/Information Units.cpd`
- `Examples/Engineering/Multiline Functions/Cubic Roots Function.cpd`
- `Examples/Engineering/Multiline Functions/Fibonacci Numbers.cpd`
- `Examples/Engineering/Multiline Functions/Greatest Common Divisor - Euclid Algorithm.cpd`
- `Examples/Engineering/Multiline Functions/Quadratic Roots Function.cpd`
