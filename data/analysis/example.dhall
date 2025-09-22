-- Example report builder in Dhall
let Rec = \(f : Type -> Type) -> forall (a : Type) -> (f a -> a) -> a

let FrameUid = { ephemeris_id : Integer, orientation_id : Integer }

let State = { from_frame : FrameUid, to_frame : FrameUid }

let VectorExprF =
      \(r : Type) ->
        < Fixed : { x : Double, y : Double, z : Double }
        | Position : State
        | Velocity : State
        | OrbitalMomentum : State
        | EccentricityVector : State
        | CrossProduct : { a : r, b : r }
        >

let VectorExpr = Rec VectorExprF

let OrbitalElement = < SemiMajorAxis : State | Eccentricity : State >

let Scalar =
      < AngleBetween : { a : VectorExpr, b : VectorExpr }
      | Norm : VectorExpr
      | OrbitalElement : OrbitalElement
      | Eclipse : { objects : List Integer }
      >

let Report =
      { epoch_start : Text
      , epoch_stop : Text
      , Data : List Scalar
      , Aberration : Text
      }

let Operation = < Eq | Ge | Geq | Le | Leq >

let When = < Rising | Falling | Either >

let Value = { scalar : Scalar, op: Operation, value : Double, precision : Double }

let Event = { evaluate : Value, when : When, time_precision_s : Double }

in  (VectorExprF VectorExpr).Position
      { from_frame = { ephemeris_id = +399, orientation_id = +0 }
      , to_frame = { ephemeris_id = +301, orientation_id = +0 }
      }
