/** WASM JSON contract types — mirror `wasm/mad-dog-sim` serde structs. */

export type SimBackend = 'wasm'

export type ModelKind = 'chain' | 'grid' | 'cube' | 'random'

export interface RunConfig {
  kind: ModelKind
  n?: number
  rows?: number
  cols?: number
  lx?: number
  ly?: number
  lz?: number
  field: number
  seed: number
}

export interface MdsResult {
  coords: number[][]
  eigenvalues: number[]
  emergentDim: number
  explainedVariance: number[]
}

export interface EmergenceReport {
  mi: number[][]
  distance: number[][]
  mds: MdsResult
}

export interface RunResult {
  label: string
  qubits: number
  energy: number
  iters: number
  expectedDim: number
  report: EmergenceReport
  truePositions: { x: number; y: number }[]
  elapsedMs: number
  backend?: SimBackend
}

export interface SpacetimeSlice {
  k: number
  t: number
  coords: number[][]
  zExpectation: number[]
  signal: number[]
  energy: number
}

export interface LightCone {
  velocity: number
  arrivals: number[]
  center: number
  dt: number
}

export interface SpacetimeResult {
  sites: number
  slices: SpacetimeSlice[]
  energyDrift: number
  lightCone?: LightCone
  /** Signal-weighted centroid track for single-defect quenches. */
  worldline?: WorldlinePoint[]
  worldlines?: [WorldlinePoint[], WorldlinePoint[]]
  elapsedMs: number
  backend?: SimBackend
}

export interface SpacetimeRunConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed?: number
}

export interface Spacetime2DConfig {
  rows: number
  cols: number
  field: number
  dt: number
  steps: number
}

export interface LightConeComparison {
  lattice: LightCone
  mi: LightCone
  miTimeAvg: LightCone
  velocityRatio: number
}

export interface LightConeCompareResult {
  spacetime: SpacetimeResult
  comparison: LightConeComparison
  elapsedMs: number
  backend?: SimBackend
}

export interface AreaLawPoint {
  size: number
  sGround: number
  sRandom: number
}

export interface RtPoint {
  entropy: number
  cut: number
  size: number
  start: number
  distFromMass?: number
}

export interface HolographyReport {
  n: number
  areaLaw: AreaLawPoint[]
  rtPoints: RtPoint[]
  rtSlope: number
  rtR2: number
}

export interface HolographyRunConfig {
  n: number
  field: number
  seed?: number
}

export interface HolographyRunResult {
  label: string
  report: HolographyReport
  elapsedMs: number
  backend?: SimBackend
}

export interface RtFit {
  rtPoints: RtPoint[]
  rtSlope: number
  rtR2: number
}

export interface DensitySweepPoint {
  count: number
  density: number
  slope: number
  r2: number
  deltaSlope: number
}

export interface RtMassReport {
  n: number
  massSite: number
  strength: number
  vacuum: RtFit
  mass: RtFit
  sweep: { strength: number; slope: number; r2: number }[]
  densitySweep?: DensitySweepPoint[]
}

export interface RtMassRunConfig {
  n: number
  field: number
  strength: number
  seed?: number
  densitySweep?: boolean
  maxMassCount?: number
}

export interface RtMassRunResult {
  label: string
  report: RtMassReport
  elapsedMs: number
  backend?: SimBackend
}

export interface RefinementBaselines {
  groundAreaGrowth: number
  randomAreaGrowth: number
}

export interface RefinementReasons {
  rtFit: boolean
  rtSlope: boolean
  areaLaw: boolean
  emergentDim: boolean
}

export interface RefinementDiagnostics {
  n: number
  rtSlope: number
  rtR2: number
  emergentDim: number
  areaGrowth: number
  areaPressure: number
  pressure: number
  needsRefinement: boolean
  reasons: RefinementReasons
}

export interface RefinementQuenchSlice {
  t: number
  step: number
  diagnostics: RefinementDiagnostics
}

export interface RefinementQuenchConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed: number
}

export interface RefinementQuenchResult {
  n: number
  field: number
  dt: number
  baselines: RefinementBaselines
  slices: RefinementQuenchSlice[]
  decouplingLag: number
  elapsedMs: number
  backend?: SimBackend
}

export interface RefinementNCompareConfig {
  n: number
  deltaN?: number
  field: number
  dt: number
  quenchStep: number
  seed: number
}

export interface RefinementNCompareResult {
  n: number
  nLarge: number
  field: number
  quenchStep: number
  dt: number
  small: RefinementDiagnostics
  large: RefinementDiagnostics
  largerRelieves: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface SplitEvent {
  triggerStep: number
  triggerT: number
  preN: number
  postN: number
  suggestedSplitSite: number
  pre: RefinementDiagnostics
  post: RefinementDiagnostics
  accepted: boolean
  pressureDelta: number
}

export interface AdaptiveRefinementConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed: number
  deltaN?: number
}

export interface AdaptiveRefinementResult {
  n: number
  deltaN: number
  field: number
  dt: number
  steps: number
  baselines: RefinementBaselines
  slices: RefinementQuenchSlice[]
  decouplingLag: number
  peakStep: number
  peakPressure: number
  splitEvent: SplitEvent | null
  elapsedMs: number
  backend?: SimBackend
}

export interface RtQuenchSlice {
  t: number
  step: number
  rtSlope: number
  rtR2: number
  meanRtRatioDev: number
  slopeDeviation: number
  areaPressure: number
}

export interface RtQuenchConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed: number
}

export interface RtQuenchResult {
  n: number
  field: number
  dt: number
  baselines: RefinementBaselines
  slices: RtQuenchSlice[]
  deviationDensityCorr: number
  structuredDeviation: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface CurvatureProxySlice {
  t: number
  step: number
  areaPressure: number
  geodesicDeviation: number
  rtSlopeDeviation: number
}

export interface CurvatureProxyQuenchConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed: number
  xi?: number
}

export interface CurvatureProxyQuenchResult {
  n: number
  field: number
  dt: number
  xi: number
  slices: CurvatureProxySlice[]
  proxyCorrelation: number
  geoDensityCorr: number
  internallyConsistent: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface PredictiveRefinementResult {
  n: number
  deltaN: number
  field: number
  dt: number
  steps: number
  baselines: RefinementBaselines
  earlyWarningStep: number | null
  failureStep: number | null
  leadTime: number
  peakStep: number
  earlySplitEvent: SplitEvent | null
  lateSplitEvent: SplitEvent | null
  earlyWarningRecoverable: boolean
  lateSplitRecoverable: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface RelationalTimeConfig {
  n: number
  field: number
  dt: number
  steps: number
  clockSite: number
  physicalSlices: number
}

export interface ClockHistory {
  label: string
  clockSite: number | null
  result: SpacetimeResult
}

export interface TimeMapPoint {
  tauUniform: number
  tauPhysical: number
}

export interface DualClockResult {
  sites: number
  uniform: ClockHistory
  physical: ClockHistory
  timeMap: TimeMapPoint[]
  syncR2: number
  syncSlope: number
  syncIntercept: number
  elapsedMs: number
  backend?: SimBackend
}

export interface MultiClockConfig {
  kind?: 'chain' | 'grid' | 'cube'
  n?: number
  rows?: number
  cols?: number
  lz?: number
  field: number
  dt: number
  steps: number
  clockSites?: number[]
  physicalSlices?: number
}

export interface PhysicalClockReading {
  site: number
  label: string
  timeMap: TimeMapPoint[]
  syncR2VsUniform: number
  syncSlopeVsUniform: number
}

export interface MultiClockResult {
  kind: string
  label: string
  sites: number
  defectSite: number
  labels: string[]
  clocks: PhysicalClockReading[]
  pairwiseR2: number[][]
  minPairwiseR2: number
  defectUniformR2: number
  edgeEdgeR2: number
  inconsistentPairs: number
  elapsedMs: number
  backend?: SimBackend
}

export interface ModularDualClockConfig {
  n: number
  field: number
  dt: number
  steps: number
  modularSlices?: number
}

export interface ModularDualClockResult {
  n: number
  center: number
  regionA: number[]
  regionB: number[]
  modularTauA: number[]
  modularTauB: number[]
  tickA: number[]
  tickB: number[]
  syncR2Modular: number
  syncR2Z: number
  syncR2ModularAUniform: number
  syncR2ModularBUniform: number
  syncR2ZAUniform: number
  syncR2ZBUniform: number
  minModularUniformR2: number
  minZUniformR2: number
  minZEdgeUniformR2: number
  elapsedMs: number
  backend?: SimBackend
}

export interface SimultaneityConfig {
  n: number
  field: number
  dt: number
  steps: number
  clockA?: number
  clockB?: number
  physicalSlices?: number
  embedDim?: number
  referenceSite?: number
}

export interface SimultaneitySlice {
  k: number
  tauA: number
  tauB: number
  tauSkew: number
  emergentX: number
  emergentY: number
}

export interface SimultaneityResult {
  sites: number
  clockA: number
  clockB: number
  referenceSite: number
  meanTauSkew: number
  maxTauSkew: number
  slopeA: number
  slopeB: number
  slopeDelta: number
  bendDetected: boolean
  slices: SimultaneitySlice[]
  elapsedMs: number
  backend?: SimBackend
}

export interface ScatteringConfig {
  n?: number
  field: number
  dt: number
  steps: number
  defectSites?: [number, number]
  /** `chain` (default), `grid`, or `cube` */
  kind?: 'chain' | 'grid' | 'cube'
  rows?: number
  cols?: number
  lx?: number
  ly?: number
  lz?: number
  lite?: boolean
  taylorOrder?: number
}

export interface LatticePosition {
  x: number
  y: number
  z?: number
}

export interface ScatteringResult {
  kind: string
  label: string
  n: number
  rows?: number
  cols?: number
  cubeDims?: [number, number, number]
  layoutPositions: LatticePosition[]
  field: number
  defectSites: [number, number]
  slices: SpacetimeSlice[]
  worldlines: [WorldlinePoint[], WorldlinePoint[]]
  separationSeries: number[]
  velocities: [number, number]
  bothMoved: boolean
  crossed: boolean
  minSeparation: number
  phaseSeries: number[]
  postInteractionPhaseStd: number
  phaseStable: boolean
  overlapStep: number
  overlapDetected: boolean
  interactionPhaseShift: number
  separationTimeDelay: number
  effectiveMass: number
  dispersionVelocityMean: number
  velocityDispersionRatio: number
  elapsedMs: number
  backend?: SimBackend
}

export interface WorldlinePoint {
  t: number
  site: number
  amplitude: number
}

export interface LorentzScalingCase {
  label: string
  rows: number
  cols: number
  speedCv: number
  passed: boolean
}

export interface LorentzScalingResult {
  cases: LorentzScalingCase[]
  covSmall: number
  covLarge: number
  covImproves: boolean
  allPassed: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface DispersionMode {
  k: number
  groupVelocity: number
  omega: number
}

export interface DispersionConfig {
  n: number
  field: number
  dt: number
  steps: number
  modes?: number
}

export interface DispersionResult {
  n: number
  field: number
  modes: DispersionMode[]
  omegaSlope: number
  omegaIntercept: number
  linearR2: number
  linearAtSmallK: boolean
  effectiveMass: number
  groupVelocityMean: number
  elapsedMs: number
  backend?: SimBackend
}

export interface BoostInvarianceConfig {
  rows: number
  cols: number
  field: number
  dt: number
  steps: number
  edgeSite?: number
}

export interface BoostInvarianceResult {
  rows: number
  cols: number
  velocityUniform: number
  velocityEdgeClock: number
  relativeDelta: number
  shapeInvariant: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface BoostFrame {
  label: string
  referenceSite: number | null
  velocity: number
}

export interface PoincareCompositeConfig {
  rows?: number
  cols?: number
  field?: number
  dt?: number
  steps?: number
  nChain?: number
  chainModes?: number
  chainField?: number
  chainDt?: number
  chainSteps?: number
}

export interface PoincareCompositeResult {
  gridRows: number
  gridCols: number
  rotationCv: number
  rotationOk: boolean
  boostFrames: BoostFrame[]
  boostCv: number
  boostOk: boolean
  dispersionLinearR2: number
  dispersionVelocityCov: number
  dispersionOk: boolean
  testsPassed: number
  testsTotal: number
  allPoincareOk: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface DecoherenceQuenchConfig {
  n: number
  field: number
  dt: number
  steps: number
  coupleStep?: number
  coupling?: number
  seed?: number
}

export interface DecoherenceSlice {
  k: number
  t: number
  signal: number[]
  envP0: number
  envP1: number
  envEntropy: number
  envCoherence: number
  mixedPeakWidth: number
  branchPeakWidth: number
  mixedEntropy: number
  branchEntropy: number
}

export interface BranchTrack {
  label: string
  envBit: number
  weight: number
  worldline: WorldlinePoint[]
}

export interface DecoherenceQuenchResult {
  chainSites: number
  envQubit: number
  coupleStep: number
  coupling: number
  slices: DecoherenceSlice[]
  mixedWorldline: WorldlinePoint[]
  branches: BranchTrack[]
  sharpenRatio: number
  branchesDistinguishable: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface Universe3DConfig {
  lx: number
  ly: number
  lz: number
  field: number
  dt: number
  steps: number
}

export interface UniverseModelSummary {
  label: string
  layout: {
    truePositions: { x: number; y: number; z?: number }[]
    expectedDim: number
  }
}

export interface Universe3DResult {
  model: UniverseModelSummary
  spacetime: SpacetimeResult
  lightCone: LightCone
  defectSite: number
  edges: [number, number][]
  siteDistances: number[]
  elapsedMs: number
  backend?: SimBackend
}

export interface UniverseSliceConfig extends Universe3DConfig {
  k: number
}

export interface UniverseSliceResult {
  report: EmergenceReport
  defectSite: number
  elapsedMs: number
  backend?: SimBackend
}

export interface CouplingEdge {
  i: number
  j: number
  dist: number
}

export interface FactorizationCandidate {
  permutation: number[]
  localityFraction: number
  miNnRatio: number
  score: number
  nonlocalTerms: number
  emergentDim: number
}

export type FactorizationKind =
  | 'shuffled_chain'
  | 'shuffled_xx_chain'
  | 'shuffled_heisenberg_chain'
  | 'shuffled_sparse_chain'
  | 'random'
  | 'shuffled_grid'
export type FactorizationInputMode = 'pauli' | 'spectrum'
export type FactorizationSearchMethod = 'exact' | 'annealing' | 'greedy'
export type FactorizationGraphKind = 'line' | 'grid'

export interface FactorizationUniquenessReport {
  equivalenceClassCount: number
  bestClassSize: number
  trueInTopK: boolean
  trueClassRank?: number
  scoreGapToSecondClass: number
}

export interface FactorizationSearchConfig {
  kind: FactorizationKind
  n: number
  field: number
  seed: number
  topK?: number
  inputMode?: FactorizationInputMode
  searchMethod?: FactorizationSearchMethod
  eigenstateCount?: number
  graphKind?: FactorizationGraphKind
  rows?: number
  cols?: number
  distanceDecay?: number
  annealingSteps?: number
  spectrumScramble?: boolean
}

export interface FactorizationSearchResult {
  label: string
  n: number
  energy: number
  baseline: FactorizationCandidate
  best: FactorizationCandidate
  topCandidates: FactorizationCandidate[]
  recoveredIdentity: boolean
  permMatchDistance?: number
  trueShuffle?: number[]
  uniqueness?: FactorizationUniquenessReport
  baselineMi: number[][]
  bestMi: number[][]
  couplingEdges: CouplingEdge[]
  inputMode: string
  scorerUsed: string
  searchMethod: string
  searchIters: number
  elapsedMs: number
  backend?: SimBackend
}

export interface FactorizationEnsembleCaseResult {
  label: string
  kind: string
  n: number
  seed: number
  recoveredIdentity: boolean
  score: number
}

export interface FactorizationEnsembleResult {
  cases: FactorizationEnsembleCaseResult[]
  recovered: number
  total: number
  recoveryRate: number
  passed: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface FactorizationRefinementConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed: number
  annealingSteps?: number
}

export interface FactorizationRefinementSlice {
  t: number
  step: number
  refinementPressure: number
  factorizationScore: number
  localityFraction: number
  permutation: number[]
  permDrift: number
  diagnostics: RefinementDiagnostics
}

export interface FactorizationRefinementResult {
  n: number
  field: number
  dt: number
  slices: FactorizationRefinementSlice[]
  initialPermutation: number[]
  elapsedMs: number
  backend?: SimBackend
}

export interface FalsificationTest {
  id: string
  name: string
  passed: boolean
  detail: string
}

export interface FalsificationBatteryResult {
  tests: FalsificationTest[]
  passed: number
  total: number
  elapsedMs: number
  backend?: SimBackend
}

export interface ExcitationSubspaceConfig {
  n: number
  field: number
  dt: number
  steps: number
  coupleStep?: number
  coupling?: number
  seed?: number
  windowRadius?: number
}

export interface PauliSiteDiagnostic {
  site: number
  mixedZ: number
  branch0Z: number
  branch1Z: number
  mixedX: number
  branch0X: number
  branch1X: number
}

export interface ExcitationSubspaceResult {
  chainSites: number
  excitationPeak: number
  windowSites: number[]
  envP0: number
  envP1: number
  mixedSharpness: number
  branch0Sharpness: number
  branch1Sharpness: number
  sharpnessGain: number
  mixedEffectiveRank: number
  branch0EffectiveRank: number
  branch1EffectiveRank: number
  rankReduction: number
  branchOverlap: number
  siteDiagnostics: PauliSiteDiagnostic[]
  codeLike: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface StabilizerGenerator {
  label: string
  weight: number
  mixedExpectation: number
  branch0Expectation: number
  branch1Expectation: number
}

export interface StabilizerSearchReport {
  windowSites: number[]
  generators: StabilizerGenerator[]
  generatorCount: number
  codeDistance: number
  codeRate: number
  stabilizerFound: boolean
}

export interface StabilizerSearchConfig extends ExcitationSubspaceConfig {}

export interface StabilizerSearchResult {
  excitation: ExcitationSubspaceResult
  stabilizer: StabilizerSearchReport
  distanceScales: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface ParticleStabilityConfig {
  n: number
  dt: number
  steps: number
}

export interface ParticleLifetimeCase {
  field: number
  localizationFraction: number
  meanPeakWidth: number
  worldlineJitter: number
}

export interface ParticleStabilityResult {
  n: number
  dt: number
  steps: number
  ordered: ParticleLifetimeCase
  disordered: ParticleLifetimeCase
  orderedLongerLived: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface BranchBornPoint {
  coupling: number
  envP0: number
  envP1: number
  weightImbalance: number
  branchOverlap: number
}

export interface BranchBornConfig {
  n: number
  field: number
  dt: number
  steps: number
  coupleStep: number
}

export interface BranchBornResult {
  n: number
  field: number
  points: BranchBornPoint[]
  imbalanceOverlapCorr: number
  entropyOverlapCorr: number
  bornConsistent: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface EftDimensionConfig extends ExcitationSubspaceConfig {}

export interface EftDimensionResult {
  windowSites: number
  measuredDofPerSite: number
  predictedDofPerSite: number
  independentDofPerSite: number
  relativeError: number
  measuredPredictedAgree: boolean
  measuredIndependentAgree: boolean
  predictedIndependentAgree: boolean
  dofAgreement: boolean
  stabilizer: StabilizerSearchReport
  elapsedMs: number
  backend?: SimBackend
}

export interface QeccIdentification {
  nPhysical: number
  kLogical: number
  dDistance: number
  codeLabel: string
  branch0CodeFidelity: number
  branch1CodeFidelity: number
  mixedCodeFidelity: number
  branchAvgFidelity: number
  fidelitySelectivity: number
  codeSubspaceFound: boolean
}

export interface QeccProbeConfig extends ExcitationSubspaceConfig {}

export interface QeccProbeResult {
  excitation: ExcitationSubspaceResult
  stabilizer: StabilizerSearchReport
  qecc: QeccIdentification
  elapsedMs: number
  backend?: SimBackend
}

export interface InplaceSplitConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed: number
  deltaN?: number
}

export interface InplaceSplitEvent {
  triggerStep: number
  splitSite: number
  preN: number
  postN: number
  pre: RefinementDiagnostics
  inPlace: RefinementDiagnostics
  rerun: RefinementDiagnostics
  inPlaceImproves: boolean
  matchesRerun: boolean
  pressureDelta: number
}

export interface InplaceSplitResult {
  n: number
  deltaN: number
  field: number
  dt: number
  steps: number
  splitEvent: InplaceSplitEvent | null
  elapsedMs: number
  backend?: SimBackend
}

export interface FactorizationCompareConfig {
  nSmall: number
  field: number
  dt: number
  step: number
  seed: number
  deltaN?: number
  splitSite?: number
}

export interface FactorizationCompareCase {
  label: string
  n: number
  localityFraction: number
  emergentDim: number
  pressure: number
}

export interface FactorizationCompareResult {
  nSmall: number
  nLarge: number
  deltaN: number
  splitSite: number
  step: number
  roundtripFidelity: number
  localityDriftEmbed: number
  localityDriftRoundtrip: number
  dimDriftEmbed: number
  dimDriftRoundtrip: number
  embeddingFaithful: boolean
  cases: FactorizationCompareCase[]
  elapsedMs: number
  backend?: SimBackend
}

export interface HolographicBoundConfig {
  field: number
  nMin?: number
  nMax?: number
}

export interface HolographicBoundPoint {
  n: number
  rtR2: number
  rtSlope: number
  emergentDim: number
  localityFraction: number
  areaLawOk: boolean
  dimOk: boolean
  localityOk: boolean
  allOk: boolean
}

export interface HolographicBoundResult {
  field: number
  points: HolographicBoundPoint[]
  nMin: number | null
  nSaturation: number
  boundScales: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface FieldSweepConfig {
  fields?: number[]
  nMin?: number
  nMax?: number
  skipLocality?: boolean
}

export interface FieldSweepPoint {
  field: number
  scan: HolographicBoundResult
}

export interface FieldSweepResult {
  points: FieldSweepPoint[]
  fieldMaxNMin: number | null
  nMinMax: number | null
  hHolographicEmergence: number | null
  emergenceNearCritical: boolean
  orderedPhaseNonholographic: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface FactorCountDynamicsConfig {
  nStart?: number
  nMax?: number
  deltaN?: number
  field?: number
  dt?: number
  steps?: number
  seed?: number
}

export interface FactorCountCandidateDiagnostics {
  n: number
  rtR2: number
  rtSlope: number
  emergentDim: number
  areaLawOk: boolean
  dimOk: boolean
  rtOk: boolean
  allOk: boolean
  pressure: number
}

export interface FactorCountPoint {
  step: number
  t: number
  candidates: FactorCountCandidateDiagnostics[]
  nMin: number | null
  nOptPressure: number
  minPressure: number
}

export interface FactorCountDynamicsResult {
  nStart: number
  nMax: number
  deltaN: number
  field: number
  dt: number
  series: FactorCountPoint[]
  nMinIncreases: boolean
  nMinInitial: number | null
  nMinFinal: number | null
  nMinPeak: number | null
  nOptPressureIncreases: boolean
  nOptPressurePeak: number
  elapsedMs: number
  backend?: SimBackend
}

export type GeometryStabilityKind = 'chain' | 'grid' | 'cube'

export interface GeometryStabilityConfig {
  kind?: GeometryStabilityKind
  n?: number
  rows?: number
  cols?: number
  lz?: number
  field: number
  dt: number
  steps: number
  xi?: number
  seed?: number
}

export interface GeometryStabilitySlice {
  k: number
  t: number
  emergentDim: number
  topEigenvalues: number[]
  distanceDrift: number
  rankCorrelation: number
  embeddingCorrelation: number
}

export interface GeometryStabilityResult {
  kind: string
  label: string
  sites: number
  field: number
  expectedDim: number
  slices: GeometryStabilitySlice[]
  meanDistanceDrift: number
  meanRankCorrelation: number
  meanEmbeddingCorrelation: number
  dimStd: number
  meanEmergentDim: number
  geometryStable: boolean
  dimStable: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface GeometryDimSweepCase {
  label: string
  kind: string
  sites: number
  expectedDim: number
  emergentDim: number
  embeddingCorrelation: number
  passed: boolean
}

export interface GeometryDimSweepResult {
  cases: GeometryDimSweepCase[]
  passed: number
  total: number
  allPassed: boolean
  elapsedMs: number
  backend?: SimBackend
}

export interface LocalitySpectrumCase {
  model: string
  n: number
  field: number
  seed: number
  recovered: boolean
  score: number
  miNnRatio: number
}

export interface LocalitySpectrumResult {
  total: number
  recovered: number
  recoveryRate: number
  pass: boolean
  cases: LocalitySpectrumCase[]
  elapsedMs: number
  backend?: SimBackend
}
