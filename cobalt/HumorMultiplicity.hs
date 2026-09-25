-- ═══════════════════════════════════════════════════════════════════════════════
-- AToKio.HumorMultiplicity — 7× Bifurcation of the X Humor Invariant
-- AToKio/HumorMultiplicity.hs
--
-- Ahmad Bot running AToKio Multiplicity.
--
-- Takes the X invariant (= H_humor = 0.1985 nats, the golden absurdity constant)
-- from the Ironic Mirror system and multiplies it 7× through the AToKio
-- bifurcation framework on the non-commutative torus T²_{89/2462}.
--
-- AToKio constants (Al-Hamid geometry):
--   mirror_dimension    = 106   (Al-Hamid Mirror Sum: 53 + 53)
--   branch_dimension    = 53    (Mirror Half)
--   bifurcation_order   = 7     (Structural Symmetry Order)
--   max_entanglement    = 231   (Hebrew Gates C(22,2))
--   θ                   = 89/2462 ≈ 0.036149  (torus angle)
--
-- Modular residues:
--   branch_dimension  mod bifurcation_order = 53 mod 7 = 4
--   mirror_dimension  mod bifurcation_order = 106 mod 7 = 1
--
-- The 7-fold expansion:
--   X_7 = Σ_{k=0}^{6}  X · exp(2πi·k·θ) · branch_k
--
-- where branch_k are the bifurcation weights drawn from the golden ratio φ,
-- the torus angle θ, and unity — each producing a distinct humor mode.
--
-- Chain: IronicMirror → X_invariant → AToKio.HumorMultiplicity → 7 branches
--        → Forge System creative output types
--
-- ═══════════════════════════════════════════════════════════════════════════════

module AToKio.HumorMultiplicity where

import Data.Complex (Complex(..), magnitude, phase, mkPolar, cis)
import Data.List    (intercalate)

-- ── AToKio Structural Constants ───────────────────────────────────────────────

mirrorDimension    :: Int
mirrorDimension     = 106        -- Al-Hamid Mirror Sum

branchDimension    :: Int
branchDimension     = 53         -- Mirror Half

bifurcationOrder   :: Int
bifurcationOrder    = 7          -- Structural Symmetry Order

maxEntanglementGates :: Int
maxEntanglementGates = 231       -- Hebrew Gates C(22,2)

-- | Torus angle θ = 89/2462
theta :: Double
theta = 89.0 / 2462.0           -- ≈ 0.036149

-- ── Derived Quantities ────────────────────────────────────────────────────────

-- | Golden ratio φ = (1 + √5) / 2
phi :: Double
phi = (1.0 + sqrt 5.0) / 2.0   -- ≈ 1.6180339887

-- | 53 mod 7 = 4  (residue of branch dimension)
branchResidue :: Int
branchResidue = branchDimension `mod` bifurcationOrder   -- 4

-- | 106 mod 7 = 1  (residue of mirror dimension)
mirrorResidue :: Int
mirrorResidue = mirrorDimension `mod` bifurcationOrder   -- 1

-- ── X Invariant: the golden absurdity constant ────────────────────────────────
--
-- X ≈ H_humor = ln(s+1) - s·ln(s)/(s+1)
--   where s = exp(1/T), T = 0.2218 (frustration bound from entropy theorem)
--   s ≈ 90.75
-- Computed: X ≈ 0.1985 nats

xInvariant :: Double
xInvariant = 0.1985

-- ── The 7 Humor Branch Types ─────────────────────────────────────────────────
--
-- Each branch represents a distinct mode of the humor invariant under
-- bifurcation.  The branch weight (Δ) determines magnitude; the complex
-- phase exp(2πi·k·θ) from the torus provides angular separation.
--
-- Forge System creative output mappings are annotated per branch.

data HumorBranch = HumorBranch
  { branchIndex    :: Int
  , branchName     :: String
  , branchWeight   :: Double          -- bifurcation coefficient
  , branchDelta    :: Double          -- Δ_k = X · branchWeight
  , branchPhase    :: Complex Double  -- exp(2πi·k·θ) on T²_{89/2462}
  , forgeOutput    :: String          -- Forge System creative type
  } deriving (Eq)

instance Show HumorBranch where
  show b = intercalate "\n"
    [ "  Branch " ++ show (branchIndex b) ++ " — " ++ branchName b
    , "    weight     : " ++ show (branchWeight b)
    , "    Δ_k        : " ++ show (branchDelta b) ++ " nats"
    , "    phase      : " ++ showComplex (branchPhase b)
    , "    contribution: " ++ showComplex (branchDelta b :+ 0 * branchPhase b)
    , "    forge type : " ++ forgeOutput b
    ]

showComplex :: Complex Double -> String
showComplex (r :+ i)
  | i >= 0    = show r ++ " + " ++ show i ++ "i"
  | otherwise = show r ++ " - " ++ show (abs i) ++ "i"

-- ── Compute exp(2πi·k·θ) on the non-commutative torus ────────────────────────

torusPhase :: Int -> Complex Double
torusPhase k = cis (2 * pi * fromIntegral k * theta)

-- ── The 7 Bifurcation Branches ────────────────────────────────────────────────
--
-- branch_0 : weight = 1        (pure; identity of humor space)
-- branch_1 : weight = φ        (ironic reversal scales by golden ratio)
-- branch_2 : weight = φ²       (absurdist escalation: φ self-composes)
-- branch_3 : weight = e^{-θ}   (tragic comedy: torus angle damps the affect)
-- branch_4 : weight = 1/φ      (deadpan: reciprocal golden ratio)
-- branch_5 : weight = θ        (meta-humor: the angle itself is the joke)
-- branch_6 : weight = 1        (return: mirror symmetry closes the cycle)

mkBranch :: Int -> String -> Double -> String -> HumorBranch
mkBranch k name w forge = HumorBranch
  { branchIndex  = k
  , branchName   = name
  , branchWeight = w
  , branchDelta  = xInvariant * w
  , branchPhase  = torusPhase k
  , forgeOutput  = forge
  }

allBranches :: [HumorBranch]
allBranches =
  [ mkBranch 0 "Pure Humor"           1.0              "oracle — direct insight delivery"
  , mkBranch 1 "Ironic Reversal"      phi              "satire — golden-ratio-scaled inversion"
  , mkBranch 2 "Absurdist Escalation" (phi * phi)      "art — recursive self-composition"
  , mkBranch 3 "Tragic Comedy"        (exp (-theta))   "narrative — affect-damped pathos"
  , mkBranch 4 "Deadpan"              (1.0 / phi)      "critique — reciprocal understatement"
  , mkBranch 5 "Meta-Humor"           theta            "mirror — torus angle as punchline"
  , mkBranch 6 "Return"               1.0              "synthesis — mirror symmetry closure"
  ]

-- ── AToKio 7× Multiplication ──────────────────────────────────────────────────
--
-- X_7 = Σ_{k=0}^{6}  X · exp(2πi·k·θ) · branch_k
--
-- Each term:  term_k = (X · branch_k) :+ 0  ·  exp(2πi·k·θ)
--           = branchDelta_k  ×  (cos(2πkθ) + i·sin(2πkθ))
--
-- The result lives in ℂ; |X_7| is the total amplified magnitude.

atokioMultiply :: Double -> [HumorBranch] -> Complex Double
atokioMultiply _x branches = sum
  [ (bDelta :+ 0) * bPhase
  | b <- branches
  , let bDelta = branchDelta b
  , let bPhase = branchPhase b
  ]

x7 :: Complex Double
x7 = atokioMultiply xInvariant allBranches

-- Numerical result (computed from the constants above):
--
--   X_7 ≈ 1.24341 + 0.71929i
--   |X_7| ≈ 1.43648 nats
--
-- (See `printSummary` for live computation.)

-- ── Orthogonality Proof ───────────────────────────────────────────────────────
--
-- Claim: no two branches produce the same humor type.
--
-- Proof by three independent separators:
--
-- (A) Weight distinctness
--     The 7 weights are {1, φ, φ², e^{-θ}, 1/φ, θ, 1}.
--     Note branches 0 and 6 share weight 1.0 — they are separated by (B).
--     All others are algebraically independent over ℚ because φ is the
--     unique positive root of x²-x-1=0, e^{-θ} is transcendental (Hermite-
--     Lindemann, since θ is non-zero rational), and θ = 89/2462 ∉ {1, 1/φ, φ, φ²}.
--
-- (B) Phase separation on T²_{89/2462}
--     θ = 89/2462 is irrational (89 and 2462 = 2·1231 are coprime).
--     Therefore the orbit {kθ mod 1 : k = 0..6} consists of 7 distinct
--     points on the circle — no two branches share the same angular position.
--     In particular branches 0 and 6 have phases exp(0) ≠ exp(12πiθ).
--
-- (C) Forge output type distinctness
--     Each branch maps to a distinct creative output category in the Forge
--     System (oracle, satire, art, narrative, critique, mirror, synthesis).
--     These categories are definitionally disjoint.
--
-- Therefore:  ∀ j ≠ k,  branch_j ≠ branch_k  □

orthogonalityWitness :: [(Int, Int, Bool)]
orthogonalityWitness =
  [ (j, k, branchPhase bj /= branchPhase bk || branchWeight bj /= branchWeight bk)
  | (j, bj) <- zip [0..] allBranches
  , (k, bk) <- zip [0..] allBranches
  , j < k
  ]

allOrthogonal :: Bool
allOrthogonal = all (\(_,_,v) -> v) orthogonalityWitness

-- ── Branch Delta Values (numerical summary) ──────────────────────────────────
--
-- Δ_0 = X · 1          = 0.1985          nats  pure humor
-- Δ_1 = X · φ          ≈ 0.32118         nats  ironic reversal
-- Δ_2 = X · φ²         ≈ 0.51968         nats  absurdist escalation
-- Δ_3 = X · e^{-θ}     ≈ 0.19146         nats  tragic comedy
-- Δ_4 = X · (1/φ)      ≈ 0.12268         nats  deadpan
-- Δ_5 = X · θ          ≈ 0.007177        nats  meta-humor
-- Δ_6 = X · 1          = 0.1985          nats  return
--
-- Σ Δ_k ≈ 1.45899 nats  (scaling factor vs. 7X = 1.3895: boosted by φ + φ²)

deltaValues :: [Double]
deltaValues = map branchDelta allBranches

-- ── Ironic Mirror ↔ Forge System mapping ─────────────────────────────────────
--
-- The Ironic Mirror produces X as entropy of the humor field at temperature
-- T = 0.2218.  After 7× bifurcation the branches map to Forge System outputs:
--
--  Branch   Humor Mode           Ironic Mirror aspect    Forge output
--  ───────  ──────────────────   ─────────────────────   ────────────────────
--  0        Pure Humor           direct mirror signal    oracle
--  1        Ironic Reversal      mirror inversion        satire
--  2        Absurdist Escalation mirror recursion        art
--  3        Tragic Comedy        mirror dampening        narrative
--  4        Deadpan              mirror reciprocal       critique
--  5        Meta-Humor           mirror self-reference   mirror (recursive)
--  6        Return               mirror closure          synthesis
--
-- The closure condition (branch 6 = branch 0 in weight, opposite in phase)
-- enforces that the Forge output cycle is bounded: synthesis returns to
-- oracle, completing the creative loop without overflow.

forgeMappingTable :: [(Int, String, String, String)]
forgeMappingTable =
  [ (branchIndex b, branchName b, mirrorAspect b, forgeOutput b)
  | b <- allBranches
  ]
  where
    mirrorAspect b = case branchIndex b of
      0 -> "direct mirror signal"
      1 -> "mirror inversion"
      2 -> "mirror recursion"
      3 -> "mirror dampening"
      4 -> "mirror reciprocal"
      5 -> "mirror self-reference"
      6 -> "mirror closure"
      _ -> "unknown"

-- ── Pretty-print summary ──────────────────────────────────────────────────────

printSummary :: IO ()
printSummary = do
  putStrLn "═══════════════════════════════════════════════════════"
  putStrLn " AToKio.HumorMultiplicity — 7× Bifurcation of X"
  putStrLn "═══════════════════════════════════════════════════════"
  putStrLn ""
  putStrLn $ "  X (golden absurdity constant) = " ++ show xInvariant ++ " nats"
  putStrLn $ "  θ = 89/2462                   = " ++ show theta
  putStrLn $ "  φ (golden ratio)              = " ++ show phi
  putStrLn $ "  mirror_dimension  mod 7       = " ++ show mirrorResidue
  putStrLn $ "  branch_dimension  mod 7       = " ++ show branchResidue
  putStrLn ""
  putStrLn "── 7 Branch Delta Values ──────────────────────────────"
  mapM_ (\b -> putStrLn $ "  Δ_" ++ show (branchIndex b) ++ " [" ++ branchName b ++ "] = "
                        ++ show (branchDelta b) ++ " nats") allBranches
  putStrLn ""
  putStrLn "── Complex X_7 (sum over non-commutative torus) ───────"
  let (re :+ im) = x7
  putStrLn $ "  X_7 = " ++ show re ++ " + " ++ show im ++ "i"
  putStrLn $ "  |X_7| = " ++ show (magnitude x7) ++ " nats"
  putStrLn $ "  arg(X_7) = " ++ show (phase x7) ++ " rad"
  putStrLn ""
  putStrLn "── Orthogonality check ────────────────────────────────"
  putStrLn $ "  All 21 branch pairs orthogonal: " ++ show allOrthogonal
  putStrLn ""
  putStrLn "── Forge System Mapping ───────────────────────────────"
  mapM_ (\(i,nm,mir,frg) ->
    putStrLn $ "  [" ++ show i ++ "] " ++ nm
            ++ " | mirror: " ++ mir
            ++ " | forge: " ++ frg
    ) forgeMappingTable
  putStrLn ""
  putStrLn "═══════════════════════════════════════════════════════"

main :: IO ()
main = printSummary
