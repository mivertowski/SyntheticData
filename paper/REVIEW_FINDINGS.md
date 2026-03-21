# DataSynth Paper — Review Findings Report

Compiled from parallel research across references, datasets, formulas, theorems, and tone.

---

## A. REFERENCES — Bibliographic Verification

### Critical Errors (must fix)

| # | Ref Key | Issue |
|---|---------|-------|
| 1 | `sharma2023synthetic` | **APPEARS FABRICATED.** No paper "Synthetic Accounting Fraud Datasets: A Survey and Benchmark" by Rohit Sharma & Ajay Kumar found in JFAR or any database. No volume/pages/DOI. Likely LLM hallucination. Must remove or replace with a real reference. |
| 2 | `arya2010inferring` | **Wrong year.** Published in **2000** (CAR vol. 17 no. 3), not 2010. |
| 3 | `lopez2018banksim` | **Wrong year and authors.** BankSim was published at EMSS **2014**, not 2018. Only 2 authors: Lopez-Rojas & Axelsson. Ahmad Elmir is from the *PaySim* paper (2016), not BankSim. |
| 4 | `dbouk2023anomaly` | **Wrong journal, year, and pages.** Paper was published in *Algorithms* (MDPI), vol. 15, no. 10, p. 385, **2022** — not AIMS Mathematics vol. 8 2023. |
| 5 | `schreyer2022federated` | **Extra author.** Bernd Reimer is NOT an author on this 2022 paper. Only Schreyer, Sattarov, and Borth. Reimer was on their 2017 paper. |
| 6 | `boersma2024complex` | **Wrong first name.** Author is **Marcel** Boersma, not Matthijs. |

### Minor Errors (should fix)

| # | Ref Key | Issue |
|---|---------|-------|
| 7 | `kotelnikov2023tabddpm` | Entry type should be `@inproceedings` (ICML), not `@article`. PMLR vol. 202, pp. 17564-17579. |
| 8 | `berti2019process` | Venue is **ICPM Demo Track 2019**, not "BPM Demonstration Track." |
| 9 | `pearl2009causality` | Entry type should be `@book`, not `@article`. |
| 10 | `ji2022survey` | Entry type should be `@article`, not `@inproceedings`. |
| 11 | `ghahfarokhi2021ocel` | Entry type should be `@inproceedings`, not `@article`. |
| 12 | `ieee_cis2019` | Dataset was provided by Vesta Corporation in partnership with IEEE-CIS. Minor attribution gap. |
| 13 | `liang2021pattern` | Author "Andi Wang" may be "Aluna Wang" (CMU slides show "Aluna"). Difficult to confirm. |
| 14 | `guo2022picture` | First author sometimes appears as "Wei (Ken) H. Guo." Very minor. |

### Verified (26 references — no issues)

benford1938law, nigrini2012benfords, xu2019modeling, patki2016synthetic, solatorio2023realtabformer, west2016intelligent, hilal2022financial, schreyer2017detection, pourhabibi2020fraud, vasarhelyi2004principles, dwork2006calibrating, xie2018differentially, jordon2019pate, sox2002, gdpr2016, nelsen2006introduction, ijiri1965generalized, ijiri1993beauty, fellingham2018double, iso21378, pei2020bigdata, rubin2005causal, kahn1962topological, bernstein2008chacha, hogan2021knowledge, hadamard1902problemes

---

## B. DATASET CLAIMS — Fact-Check

### 1. GL.ai / `schreyer2022federated` — MAJOR PROBLEM

**Paper claims:** "The GL.ai dataset provides federated synthetic journal entries but lacks document-flow linkage and subledger coherence."

**Reality:** GL.ai is **PwC's proprietary commercial audit tool** for anomaly detection in general ledger data. It is NOT a dataset and NOT from the cited Schreyer et al. paper. The cited paper (arXiv:2208.12708) proposes a federated learning framework — it does not produce or distribute a dataset called "GL.ai."

**Action needed:** Remove the GL.ai name and rewrite this entry. Either (a) describe what Schreyer et al. 2022 actually does, or (b) find and cite the actual source if a different dataset was intended.

### 2. SAFD / `sharma2023synthetic` — DOES NOT EXIST

**Paper claims:** "The SAFD (Simulated Accounting Fraud Dataset) generates entries with injected anomalies, yet uses uniform random amounts that violate Benford's law."

**Reality:** No evidence this paper or dataset exists anywhere. The "SAFD" acronym has no matches in accounting/fraud literature. The claim about "uniform random amounts" cannot be verified.

**Action needed:** Remove entirely. Replace with a real dataset/paper if one exists that makes the same point, or remove the row from the comparison table.

### 3. BankSim / `lopez2018banksim` — CLAIM ACCURATE, CITATION WRONG

**Paper claims:** "BankSim focuses narrowly on payment transactions for fraud research."

**Reality:** Claim is accurate. BankSim generates synthetic payment transactions between customers and merchants with injected fraud. No journal entries, no accounting context.

**Citation issues:** Year should be 2014 (not 2018), author list should be Lopez-Rojas & Axelsson only (no Elmir).

### 4. IEEE-CIS / `ieee_cis2019` — ACCURATE

**Paper claims:** "provides real transaction labels but lacks accounting context."

**Reality:** Correct. 590,540 e-commerce transactions with binary fraud labels, 431 mostly anonymized features, no accounting structure.

### 5. Comparison Table (`tab:related_comparison`)

- **GL.ai row:** Misleading — treats GL.ai as a data generation system when it's a detection tool. The checkmarks (Bal, Anom) are not meaningful.
- **SAFD row:** Based on non-existent reference. Must be removed.
- **Other rows:** Appear reasonable.

---

## C. FORMULAS — Mathematical Validation

### Errors Found

| # | Equation | Location | Issue | Severity |
|---|----------|----------|-------|----------|
| 1 | `eq:stirling_matching` | ground_truth.tex:56 | **INCORRECT.** Formula has spurious n!/m! prefactor. The number of surjections from n-set to m-set is `Sum_{k=0}^{m} (-1)^{m-k} C(m,k) k^n` — no n! factor. The paper's formula gives A(4,3)=144, but the text claims A(4,3)=36. The correct answer IS 36, so the formula contradicts its own example. | **HIGH** |
| 2 | Proof algebra | ground_truth.tex:80 | **GARBLED.** "n! · S(n,m) / S(n,m) · m! · S(n,m) = n! / m! · Σ..." is nonsensical. Must be completely rewritten. | **HIGH** |
| 3 | Prop 4.1 MAD bound | stat_foundations.tex:58 | **INCORRECT.** Claimed E[MAD] ≤ (1/9)√(π/(2n)) is violated by factor ~1.7x when you compute with actual Benford probabilities. However, the practical conclusion (n≥7000 gives MAD<0.006) still holds — actual E[MAD] ≈ 0.00281 at n=7000. | **MEDIUM** |
| 4 | Log-normal mean table | stat_foundations.tex:131-133 | **Two values wrong.** "Routine" mean should be ~$1,245 (not $1,224). "Major" mean should be ~$82,432 (not $89,352). The formula e^{μ+σ²/2} is correct but the computed numbers don't match. | **MEDIUM** |
| 5 | `eq:period_end` (extended crunch) | stat_foundations.tex:165 | **Singularity at t=T.** Formula diverges to infinity at period end. Also behavior at t=0 is inverted compared to exponential case. Needs a clamping note or formula fix. | **LOW** |
| 6 | `eq:temporal_graph` | knowledge_graph.tex:20 | **Nonstandard notation.** G = A × E uses Cartesian product symbol for "graph with nodes A and edges E." Should be G = (A, E) tuple notation. | **LOW** |
| 7 | `eq:difficulty` | anomaly_injection.tex:109 | **Incomplete.** States d ∈ [0,1] but doesn't constrain that weights w_f sum to 1 or that each s_f ∈ [0,1]. | **LOW** |

### Correct (no issues)

- `eq:error_persistence` — correct; table values verified
- `eq:amplification` — correct as a model
- `eq:benford_first` — standard Benford formula
- `eq:benford_two` — standard two-digit generalization
- `eq:lognormal_mixture` — correct density formula
- `eq:drift` — sensible piecewise regime model
- `eq:dp_laplace` — standard Laplace mechanism
- Prop 8.1 utility bound — correct (R/(nε))
- `eq:severity` — weights sum to 1.00
- `eq:completeness` — correct by construction

### Notation Conflicts

| Symbol | Conflicting meanings | Severity |
|--------|---------------------|----------|
| **ε** | Error detection rate (§3) vs. DP privacy budget (§8) vs. matching tolerance (§5) | **HIGH** |
| **G** | Generative process (§3) vs. temporal graph (§6) vs. causal DAG (§9) | **HIGH** |
| **α** | Decay parameter (§4) vs. propagation factor (§3) vs. intervention onset (§9) vs. significance level (§10) | **MEDIUM** |
| **n** | Credit line items (§3) vs. sample size (§4) vs. record count (§8) vs. DAG nodes (§9) | **MEDIUM** |
| **d** | Benford digit (§4) vs. difficulty score (§7) vs. copula dimension (§4) vs. dimension count (§3) | **MEDIUM** |
| **K** | Knowledge graph (§3) vs. mixture components (§4) | **LOW** (calligraphic vs italic) |

---

## D. THEOREMS — Necessity and Correctness

| Theorem | Assessment | Recommendation |
|---------|-----------|----------------|
| **Thm 3.1** (Combinatorial Infeasibility) | Known combinatorics result (surjection count) applied to accounting. Formula is wrong (see §C). Proof is garbled. Also: the surjection model only captures discrete assignment, not continuous amount-splitting (which has uncountably many solutions). This gap is never acknowledged. | **REWRITE.** Fix formula. Fix proof. Acknowledge discrete-vs-continuous simplification. Consider downgrading to Proposition since the math is a known result. |
| **Cor 3.2** | Conflates enumeration infeasibility with verification infeasibility. You don't need to enumerate all configs to verify one — you'd just need additional information to pick the right one. | **REWRITE.** Separate the two claims. |
| **Prop 3.3** (Error Persistence) | Mathematically correct. Independence assumption is optimistic (real detection is correlated, so actual persistence is higher). | **KEEP.** Add remark that independence is best-case. |
| **Thm 3.4** (Cross-Dimensional Amplification) | The "proof" is just one example. Multiplicative model is asserted without justification. "Independence of propagation paths" is claimed but not established. | **DOWNGRADE to Observation.** Replace "Proof" with "Example" or "Discussion." |
| **Thm 3.6** (Completeness) | Tautological: "if the model specifies everything, the output contains everything." The "proof" is circular. Polynomial decidability claim is trivially true for any finite dataset. | **DOWNGRADE to Design Property or Observation.** |
| **Prop 4.1** (MAD bound) | Bound formula is numerically wrong (off by ~1.7x). Practical threshold (n≥7000) still holds. No derivation or reference given. | **FIX formula** or cite a reference. The practical conclusion can be kept. |
| **Prop 8.1** (Utility bound) | Correct. Standard DP result. | **KEEP AS-IS.** |

---

## E. TONE — Passages Requiring Revision

### Pattern 1: Overuse of "impossible" / absolute language

| Location | Current text | Issue |
|----------|-------------|-------|
| abstract.tex:4-6 | "built by scraping and mining existing operational data" | "Scraping and mining" sounds crude/dismissive |
| abstract.tex:7-9 | "solving a jigsaw puzzle without ever having seen the picture on the box" | Implies practitioners work blindly; overstates degree of ignorance |
| abstract.tex:10-13 | "treating symptoms while leaving root causes undiagnosed" | Implies current audit is superficial |
| ground_truth.tex:18 | "the impossibility of reliably recovering ground truth" | "Impossibility" too strong — paper shows exhaustive recovery is infeasible, not all recovery |
| anomaly_injection.tex:8-9 | "ground-truth labels that are impossible to obtain from real enterprise data" | Partial labels exist from confirmed fraud cases, internal audit findings |
| anomaly_injection.tex:140-143 | "perfect ground truth — unlike labels derived from real data, which are inherently incomplete" | "Perfect" is overclaim — perfect w.r.t. the model, not reality |
| architecture.tex:136-138 | "a property that is impossible to achieve with inverse-recovery approaches" | Some provenance is recoverable from real data |

### Pattern 2: Dismissive characterization of practitioners/prior work

| Location | Current text | Issue |
|----------|-------------|-------|
| intro.tex:27-30 | "Teams assemble transactional fragments into knowledge graphs, hoping that the emergent structure reveals meaningful patterns. When the patterns appear consistent, they assume correctness." | "Hoping" and "assume" portray practitioners as naive. They apply professional judgment. |
| intro.tex:37 | "systematic errors that are invisible to the teams analyzing them" | Implies teams are unable to see what's in front of them |
| intro.tex:53-54 | "overwhelms traditional audit procedures" | Dismissive of current professional practice |
| related_work.tex:30-36 | "all these approaches assume the input data is a faithful representation" | Implies cited authors were naive about data quality |
| related_work.tex:74-77 | "They generate isolated data artifacts rather than interconnected reference knowledge" | "Isolated data artifacts" dismisses prior work that served valid purposes |

### Pattern 3: Replacement framing (DataSynth supersedes rather than complements)

| Location | Current text | Issue |
|----------|-------------|-------|
| conclusion.tex:7-13 | "building knowledge systems by scraping existing enterprise data — the dominant approach" | Repeats dismissive "scraping" framing |
| ground_truth.tex:310-322 | Forward vs. Inverse comparison table | Inverse column is uniformly negative; no acknowledgment that real data provides contextual richness |
| knowledge_graph.tex:29-32 | "Unlike accounting networks constructed from real data via inverse recovery" | "Unlike" sets up adversarial contrast |

### Suggested Rewording Strategy

1. **Replace "impossible"** with "difficult to achieve comprehensively" or "inherently limited"
2. **Soften the jigsaw metaphor** from "without the picture" to "with only a partial view of the picture" — acknowledges domain expertise as a partial guide
3. **Replace "scraping and mining"** with "constructed from" or "derived from"
4. **Replace "hoping" / "assume"** with "applying professional judgment" + "however, internal consistency alone cannot guarantee..."
5. **Replace "overwhelms"** with "creates complexity that is difficult to address through any single analytical approach"
6. **Replace "all these approaches assume"** with "these approaches generally rely on available data as their starting point"
7. **Replace "isolated data artifacts"** with "were designed to address specific aspects of the synthetic data challenge"
8. **Replace "Unlike"** with "Complementing" when contrasting with real-data approaches
9. **Add qualifying language** from §3.5 ("does not replace the need to analyze real enterprise data") more prominently in abstract, intro, and conclusion
10. **Nuance the comparison table:** "Unknown; inferred from data" → "Partially recoverable via domain expertise"

---

## F. IMPLEMENTATION BOASTING — Lines to Remove/Rewrite

| Location | Text | Action |
|----------|------|--------|
| abstract.tex:45 | "Implemented in Rust across 16 crates totaling over 120,k lines of code" | **Remove** LoC count and crate count. Keep that it's implemented in Rust if desired. |
| abstract.tex:52-53 | "33-type anomaly injection framework" | **Simplify** — the type count doesn't add scientific value in the abstract |
| intro.tex:102 | "comprising 16 Rust crates" | **Remove** crate count |
| architecture.tex:49 | "over 200 domain model types" | **Remove** or say "comprehensive domain model types" |
| architecture.tex:75 | "50+ specialized generators" | **Remove** count |
| Various (abstract, intro, arch, KG, conclusion) | "78+ entity types and 39+ relationship types" | **Keep in ONE place** (knowledge_graph section) where it's substantive. Remove from abstract, intro, conclusion. |
| Various | "20+ enterprise processes" | **Keep in ONE place** (generation_pipeline). Remove repetitions. |
| Various | "15+ coherence validators" | **Keep in ONE place** (evaluation). Remove repetitions. |
| conclusion.tex:38-42 | Repeats throughput, process count, and provenance claims | **Trim** to essentials — the experiments section already provides evidence. |

**Keep:** All throughput numbers backed by experiments tables (§11.5), Benford MAD scores (§11.2), F1 scores (§11.4).

---

## Summary of Action Items by Priority

### CRITICAL (will cause reviewers to reject)
1. Remove or replace `sharma2023synthetic` (SAFD) — likely fabricated reference
2. Fix GL.ai misattribution — it's PwC's tool, not the cited paper
3. Fix `eq:stirling_matching` — formula is mathematically wrong
4. Fix garbled proof algebra in Theorem 3.1

### HIGH (significant errors)
5. Fix `arya2010inferring` year: 2000, not 2010
6. Fix `lopez2018banksim`: year 2014, remove Elmir from authors
7. Fix `dbouk2023anomaly`: journal is Algorithms (MDPI) 2022, not AIMS Math 2023
8. Fix `schreyer2022federated`: remove Reimer from author list
9. Fix `boersma2024complex`: Marcel, not Matthijs
10. Fix Prop 4.1 MAD bound formula
11. Fix log-normal mean table values ($1,245 and $82,432)
12. Tone overhaul — 18 passages flagged across 9 sections

### MEDIUM (improve rigor)
13. Downgrade Thm 3.4 to Observation
14. Downgrade Thm 3.6 to Design Property
15. Rewrite Corollary 3.2 to separate enumeration from verification
16. Resolve ε and G notation conflicts
17. Remove LoC count and repeated implementation-scale claims
18. Update comparison table (remove SAFD row, fix GL.ai row)

### LOW (polish)
19. Fix BibTeX entry types (kotelnikov, pearl, ji, ghahfarokhi)
20. Fix PM4Py venue (ICPM not BPM)
21. Add clamping note for extended crunch formula
22. Add weight constraint to difficulty equation
23. Fix graph notation G=(A,E) not G=A×E
