# Paper Quality Overhaul — Implementation Plan

> **For agentic workers:** Execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. All edits are to LaTeX/BibTeX files in `paper/`. After all tasks, recompile and verify.

**Goal:** Fix all identified issues in the DataSynth paper: fabricated/wrong references, mathematical errors, theorem over-formalization, tone issues, and implementation boasting.

**Architecture:** Seven task groups executed in dependency order: (1) critical reference fixes, (2) remaining reference fixes, (3) formula corrections, (4) theorem restructuring, (5) tone overhaul, (6) implementation boasting removal, (7) comparison table and final consistency pass.

**Files touched:**
- `paper/references.bib` — Tasks 1, 2
- `paper/sections/related_work.tex` — Tasks 1, 5, 7
- `paper/sections/ground_truth.tex` — Tasks 3, 4, 5
- `paper/sections/statistical_foundations.tex` — Tasks 3
- `paper/sections/anomaly_injection.tex` — Tasks 3, 5
- `paper/sections/knowledge_graph.tex` — Tasks 3, 5
- `paper/sections/abstract.tex` — Tasks 5, 6
- `paper/sections/introduction.tex` — Tasks 5, 6
- `paper/sections/architecture.tex` — Tasks 5, 6
- `paper/sections/privacy.tex` — Tasks 3
- `paper/sections/conclusion.tex` — Tasks 5, 6
- `paper/sections/evaluation.tex` — Task 7
- `paper/sections/counterfactual.tex` — Task 7

---

### Task 1: Critical Reference Fixes (references.bib + related_work.tex)

**Files:**
- Modify: `paper/references.bib`
- Modify: `paper/sections/related_work.tex`

- [ ] **Step 1: Remove `sharma2023synthetic` (SAFD) from references.bib**

Delete the entire bib entry for `sharma2023synthetic` (the paper appears fabricated — no trace in any database).

- [ ] **Step 2: Remove all SAFD references from related_work.tex**

In `related_work.tex` line 68-70, remove: "The SAFD (Simulated Accounting Fraud Dataset)~\citep{sharma2023synthetic} generates entries with injected anomalies, yet uses uniform random amounts that violate Benford's law."

Replace with text about a real limitation of existing accounting data synthesis — e.g., that most existing approaches to accounting data synthesis focus on individual tables without cross-table coherence.

- [ ] **Step 3: Fix GL.ai / `schreyer2022federated` misattribution**

In `related_work.tex` lines 67-68, the text says "The GL.ai dataset~\citep{schreyer2022federated} provides federated synthetic journal entries but lacks document-flow linkage and subledger coherence."

Rewrite to accurately describe what Schreyer et al. 2022 actually does: it proposes a federated learning framework for training anomaly detection models on journal entry data across audit clients, using real-world city payment datasets. It does not generate synthetic data or produce a dataset. Remove the "GL.ai" name (that is PwC's proprietary tool, unrelated to this paper).

New text:
```latex
\citet{schreyer2022federated} propose a federated learning framework
for training anomaly detection models on journal entry data across
audit engagements, addressing data-sharing constraints but not the
generation of synthetic reference data.
```

- [ ] **Step 4: Fix `schreyer2022federated` author list in references.bib**

Remove "Reimer, Bernd" from the author list. The 2022 paper has only three authors: Schreyer, Sattarov, and Borth. Also update venue from arXiv to ICAIF 2022:

```bibtex
@inproceedings{schreyer2022federated,
  title={Federated and Privacy-Preserving Learning of Accounting Data
    in Financial Statement Audits},
  author={Schreyer, Marco and Sattarov, Timur and Borth, Damian},
  booktitle={Proceedings of the Third ACM International Conference on
    AI in Finance (ICAIF)},
  pages={105--113},
  year={2022}
}
```

- [ ] **Step 5: Fix `lopez2018banksim` — year and authors**

Change year from 2018 to 2014. Remove Ahmad Elmir (he is from PaySim, not BankSim):

```bibtex
@inproceedings{lopez2014banksim,
  title={{BankSim}: A bank payments simulator for fraud detection research},
  author={Lopez-Rojas, Edgar Alonso and Axelsson, Stefan},
  booktitle={26th European Modeling and Simulation Symposium (EMSS)},
  year={2014}
}
```

Update the cite key from `lopez2018banksim` to `lopez2014banksim` and update all `\citep{lopez2018banksim}` references in the paper to `\citep{lopez2014banksim}`.

- [ ] **Step 6: Fix `arya2010inferring` — year is 2000, not 2010**

```bibtex
@article{arya2000inferring,
  ...
  year={2000},
  ...
}
```

Update cite key and all references from `arya2010inferring` to `arya2000inferring`.

- [ ] **Step 7: Fix `dbouk2023anomaly` — wrong journal, year, pages**

```bibtex
@article{dbouk2022anomaly,
  title={Anomaly detection in financial time series by principal
    component analysis and neural network classifiers},
  author={Dbouk, Wassim and Jamali, Ibrahim},
  journal={Algorithms},
  volume={15},
  number={10},
  pages={385},
  year={2022},
  publisher={MDPI}
}
```

Update cite key and all references.

- [ ] **Step 8: Fix `boersma2024complex` — Marcel, not Matthijs**

```bibtex
@phdthesis{boersma2024complex,
  title={Complex Networks in Audit: A Data-Driven Modelling Approach},
  author={Boersma, Marcel},
  school={Universiteit van Amsterdam},
  year={2024}
}
```

- [ ] **Step 9: Verify all cite keys compile**

Search all .tex files for any remaining references to old cite keys (lopez2018banksim, arya2010inferring, dbouk2023anomaly, sharma2023synthetic) and update them.

---

### Task 2: Minor Reference Fixes (references.bib)

**Files:**
- Modify: `paper/references.bib`
- Modify: `paper/sections/related_work.tex` (for berti venue fix)

- [ ] **Step 1: Fix `kotelnikov2023tabddpm` entry type**

Change from `@article` to `@inproceedings`, change `journal` to `booktitle`, add pages:

```bibtex
@inproceedings{kotelnikov2023tabddpm,
  title={{TabDDPM}: Modelling Tabular Data with Diffusion Models},
  author={Kotelnikov, Akim and Baranchuk, Dmitry and Rubachev, Ivan
    and Babenko, Artem},
  booktitle={Proceedings of the 40th International Conference on Machine
    Learning (ICML)},
  volume={202},
  pages={17564--17579},
  year={2023},
  series={PMLR}
}
```

- [ ] **Step 2: Fix `berti2019process` venue**

Change `booktitle` from "BPM Demonstration Track" to "ICPM 2019 Demonstration Track":

```bibtex
@inproceedings{berti2019process,
  title={{PM4Py}: A Process Mining Library for {Python}},
  author={Berti, Alessandro and van Zelst, Sebastiaan J.
    and van der Aalst, Wil M.P.},
  booktitle={ICPM 2019 Demonstration Track},
  year={2019}
}
```

- [ ] **Step 3: Fix `pearl2009causality` entry type**

Change from `@article` to `@book`, remove `journal` field (already has `publisher`):

```bibtex
@book{pearl2009causality,
  author = {Pearl, Judea},
  title = {Causality: Models, Reasoning, and Inference},
  publisher = {Cambridge University Press},
  year = {2009},
  edition = {2nd}
}
```

- [ ] **Step 4: Fix `ji2022survey` entry type**

Change from `@inproceedings` to `@article`:

```bibtex
@article{ji2022survey,
  title={A Survey on Knowledge Graphs: Representation, Acquisition,
    and Applications},
  author={Ji, Shaoxiong and Pan, Shirui and Cambria, Erik
    and Marttinen, Pekka and Yu, Philip S.},
  journal={IEEE Transactions on Neural Networks and Learning Systems},
  volume={33},
  number={2},
  pages={494--514},
  year={2022}
}
```

- [ ] **Step 5: Fix `ghahfarokhi2021ocel` entry type**

Change from `@article` to `@inproceedings`, change `journal` to `booktitle`:

```bibtex
@inproceedings{ghahfarokhi2021ocel,
  title={{OCEL}: A Standard for Object-Centric Event Logs},
  author={Ghahfarokhi, Anahita Farhang and Park, Gyunam
    and Berti, Alessandro and van der Aalst, Wil M.P.},
  booktitle={European Conference on Advances in Databases and Information
    Systems (ADBIS)},
  pages={169--175},
  year={2021},
  publisher={Springer}
}
```

---

### Task 3: Formula Corrections

**Files:**
- Modify: `paper/sections/ground_truth.tex`
- Modify: `paper/sections/statistical_foundations.tex`
- Modify: `paper/sections/anomaly_injection.tex`
- Modify: `paper/sections/knowledge_graph.tex`
- Modify: `paper/sections/privacy.tex`

- [ ] **Step 1: Fix eq:stirling_matching in ground_truth.tex**

Replace the formula (line 56-59) with the correct surjection count:

```latex
\mathcal{A}(n, m) = \sum_{k=0}^{m} (-1)^{m-k} \binom{m}{k} k^n
```

This equals m! * S(n,m) where S(n,m) is the Stirling number of the second kind. For n=4, m=3: A(4,3) = 81 - 48 + 3 = 36. This matches the text's claim.

- [ ] **Step 2: Fix the garbled proof algebra in ground_truth.tex**

Replace line 80 ("The total count of surjections is n! * S(n,m) / S(n,m)...") with:

```latex
The number of surjections from an $n$-element set to an $m$-element
set is $m! \cdot S(n,m)$ where the Stirling number of the second kind
is $S(n,m) = \frac{1}{m!}\sum_{k=0}^{m}(-1)^{m-k}\binom{m}{k}k^n$.
Multiplying through gives $\mathcal{A}(n,m) = \sum_{k=0}^{m}(-1)^{m-k}\binom{m}{k}k^n$,
which is \Cref{eq:stirling_matching}.
```

Also add a remark acknowledging the discrete-vs-continuous simplification:

```latex
\begin{remark}
\Cref{thm:combinatorial} counts discrete credit-to-debit
\emph{assignments}.  When amounts must also be split across edges,
the configuration space becomes continuous and uncountably infinite,
making the discrete count a lower bound on the true ambiguity.
\end{remark}
```

- [ ] **Step 3: Fix Proposition 4.1 MAD bound in statistical_foundations.tex**

Replace the bound formula (line 58-61):

Old: `$\E[\MAD] \leq \frac{1}{9}\sqrt{\frac{\pi}{2n}}$`

New: Replace with the correct derivation or a weaker but correct bound. The actual expected MAD is approximately (1/9) * sqrt(2/(pi*n)) * sum_{d=1}^{9} sqrt(p_d(1-p_d)) where the sum evaluates to approximately 2.65. A clean correct bound is:

```latex
$\E[\MAD] \leq \frac{1}{9}\sum_{d=1}^{9}
\sqrt{\frac{2\,p_d(1-p_d)}{\pi\,n}}
\;\leq\; \frac{1}{9}\sqrt{\frac{9}{2\pi\,n}}
\;=\; \frac{1}{3}\sqrt{\frac{1}{2\pi\,n}}$
```

using the Cauchy-Schwarz inequality. For n = 7000 this gives approximately 0.0063, so adjust the threshold claim to n >= 8000 for MAD < 0.006, or use the tighter numerical bound: "For the specific Benford probabilities, numerical evaluation gives $\E[\MAD] \approx 0.235/\sqrt{n}$, ensuring close conformity (MAD < 0.006) for $n \gtrsim 1{,}600$."

- [ ] **Step 4: Fix log-normal mean table values in statistical_foundations.tex**

Replace the table (lines 131-133):
- Routine: e^{6.0 + 1.125} = e^{7.125} ≈ **$1,245** (not $1,224)
- Significant: e^{9.0} ≈ $8,103 (correct, keep)
- Major: e^{11.0 + 0.32} = e^{11.32} ≈ **$82,269** (not $89,352)

```latex
Routine     & 0.60 & 6.0 & 1.5 & \$1{,}245 \\
Significant & 0.30 & 8.5 & 1.0 & \$8{,}103 \\
Major       & 0.10 & 11.0 & 0.8 & \$82{,}269 \\
```

- [ ] **Step 5: Add clamping note for extended crunch formula in statistical_foundations.tex**

After eq:period_end (line 167), add a note:

```latex
In the extended-crunch model the intensity diverges as $t \to T$;
in practice, \system{} clamps $\lambda(t)$ at $\lambda_0 \cdot
m_{\text{peak}}$ and truncates the active window to
$[T + \text{start\_day},\; T]$.
```

- [ ] **Step 6: Fix graph notation in knowledge_graph.tex**

Replace line 20-21:
```latex
G(t_0, t_1) = A(t_0, t_1) \times E(t_0, t_1)
```
with:
```latex
G(t_0, t_1) = \bigl(A(t_0, t_1),\; E(t_0, t_1)\bigr)
```

- [ ] **Step 7: Add weight normalization constraint to eq:difficulty in anomaly_injection.tex**

After the difficulty equation (line 113), add:
```latex
where $w_f \geq 0$, $\sum_f w_f = 1$, and each factor score $s_f \in [0, 1]$.
```

- [ ] **Step 8: Resolve epsilon notation conflict**

Throughout the paper, use distinct symbols:
- `\varepsilon` (curly epsilon) for **DP privacy budget** in privacy.tex — already used
- `\varepsilon_d` for **error detection rate** in ground_truth.tex — rename the plain epsilon in eq:error_persistence and surrounding text
- Keep `\epsilon_q`, `\epsilon_p` for matching tolerances in generation_pipeline.tex (already subscripted, less confusing)

- [ ] **Step 9: Resolve G notation conflict**

Use distinct symbols:
- `\mathcal{G}` for the **generative process** in ground_truth.tex (already calligraphic in some places — make consistent)
- `G(t_0,t_1)` for the **temporal accounting graph** in knowledge_graph.tex (keep as is)
- Rename the **causal DAG** in counterfactual.tex from `G = (V, E, \tau, w)` to `\mathcal{C} = (V, E, \tau, w)` and update all references in that section

---

### Task 4: Theorem Restructuring

**Files:**
- Modify: `paper/sections/ground_truth.tex`
- Modify: `paper/sections/statistical_foundations.tex`

- [ ] **Step 1: Consider downgrading Theorem 3.1 to Proposition**

Since the mathematical content is a known combinatorics result applied to a new domain, change:
```latex
\begin{theorem}[Combinatorial Infeasibility of Ground Truth Recovery]
```
to:
```latex
\begin{proposition}[Combinatorial Infeasibility of Ground Truth Recovery]
```
and `\end{theorem}` to `\end{proposition}`. Update the label if needed.

(Note: this is a judgment call — if the author prefers to keep it as a theorem because the framing contribution is novel, that's defensible. But the proof must be fixed regardless.)

- [ ] **Step 2: Rewrite Corollary 3.2**

The current corollary conflates enumeration with verification. Rewrite to:

```latex
\begin{corollary}
\label{cor:verification}
For any enterprise dataset with a non-trivial fraction of multi-line
journal entries, enumerating all consistent accounting network
configurations is computationally infeasible.  Consequently, without
additional information beyond the journal entries themselves,
distinguishing the true configuration from the exponentially many
alternatives is not possible by exhaustive search.
\end{corollary}
```

- [ ] **Step 3: Downgrade Theorem 3.4 to Observation**

Change:
```latex
\begin{theorem}[Cross-Dimensional Error Amplification]
```
to:
```latex
\begin{observation}[Cross-Dimensional Error Amplification]
```

Change `\end{theorem}` to `\end{observation}`.

Replace `\begin{proof}` / `\end{proof}` with a paragraph starting "**Illustrative example.**" to avoid claiming a proof for what is really an example:

```latex
\paragraph{Illustrative example.}
Consider a vendor master data error ...
[keep existing content but remove \begin{proof}/\end{proof} tags]
... The multiplicative model provides a useful upper-bound heuristic
for estimating cross-dimensional impact, though actual propagation
depends on the specific error type and process dependencies.
```

- [ ] **Step 4: Downgrade Theorem 3.6 to a Design Property**

Change:
```latex
\begin{theorem}[Reference Knowledge Graph Completeness]
```
to:
```latex
\begin{proposition}[Reference Knowledge Graph Completeness]
```

Rewrite the proof to be more honest about what it establishes:

```latex
\begin{proof}[Justification]
By construction, every record in $D$ is produced by a deterministic
function of the seed and the generative model $\mathcal{G}$.  The
structural rules $\mathcal{R}_S$ determine which entities exist and
how they relate (Layer~1).  The statistical parameters
$\Theta_\Sigma$ determine all distributional properties (Layer~2).
The normative constraints $\mathcal{C}_N$ are explicitly encoded in
the generation process, so every violation or conformance is known
(Layer~3).  This is a direct consequence of forward generation from
a fully specified model: since the model is known, any question about
the data that depends only on the model's specification is answerable
by inspecting the model.
\end{proof}
```

- [ ] **Step 5: Add remark to Proposition 3.3 about independence assumption**

After the proof of Proposition 3.3, add:

```latex
\begin{remark}
The independence assumption in \Cref{prop:error_persistence} is
optimistic: in practice, detection capabilities are often correlated
across stages (e.g., a subtle classification error that evades one
stage is likely to evade similar checks at subsequent stages).  The
actual persistence probability for systematic errors may therefore
be \emph{higher} than $(1 - \varepsilon_d)^{k-1}$, making this a
conservative estimate of the problem's severity.
\end{remark}
```

---

### Task 5: Tone Overhaul

**Files:**
- Modify: `paper/sections/abstract.tex`
- Modify: `paper/sections/introduction.tex`
- Modify: `paper/sections/ground_truth.tex`
- Modify: `paper/sections/related_work.tex`
- Modify: `paper/sections/anomaly_injection.tex`
- Modify: `paper/sections/architecture.tex`
- Modify: `paper/sections/knowledge_graph.tex`
- Modify: `paper/sections/conclusion.tex`

The guiding principle: shift from "current practice is broken" to "here is a complementary capability that addresses an inherent limitation." Acknowledge that practitioners bring expertise, that existing tools work within their scope, and that DataSynth provides reference data, not replacement methodology.

- [ ] **Step 1: abstract.tex — soften framing**

Line 4-6: Replace "built by scraping and mining existing operational data" with "typically constructed from existing operational data"

Line 7-9: Replace "solving a jigsaw puzzle without ever having seen the picture on the box" with "assembling a jigsaw puzzle with only a partial view of the picture on the box"

Line 10-13: Replace "treating symptoms while leaving root causes undiagnosed" with "potentially obscuring root causes that audit procedures seek to identify"

- [ ] **Step 2: introduction.tex — respect practitioners**

Line 27-30: Replace "Teams assemble transactional fragments into knowledge graphs, hoping that the emergent structure reveals meaningful patterns. When the patterns appear consistent, they assume correctness." with "Teams assemble transactional fragments into knowledge graphs, applying professional judgment and domain expertise to interpret the emergent structure. However, when internally consistent patterns emerge, distinguishing genuine correctness from the appearance of coherence in systematically biased data remains challenging."

Line 37: Replace "systematic errors that are invisible to the teams analyzing them" with "systematic errors that may remain undetected due to inherent information constraints in the available data"

Line 53-54: Replace "overwhelms traditional audit procedures" with "creates a combinatorial complexity that is difficult to address comprehensively through any single analytical approach"

- [ ] **Step 3: ground_truth.tex — temper absolutism**

Line 18: Replace "the impossibility of reliably recovering ground truth" with "the inherent limitations of recovering complete ground truth from observed enterprise data alone"

Lines 310-322 (comparison table): Nuance the "Inverse Recovery" column:
- "Unknown; inferred from data" → "Partially recoverable via domain expertise and corroborating evidence"
- "Inherited and amplified" → "May be inherited if undetected"
- "Partial; limited by observability" → "Rich in context but limited by observability"
- "Cannot verify against unknown truth" → "Verification constrained by data availability"
- "Requires manual annotation" → "Typically requires manual annotation"
- "Depends on data access" → "Depends on data access and agreements"

- [ ] **Step 4: related_work.tex — acknowledge prior work's value**

Lines 30-36: Replace "However, all these approaches assume the input data is a faithful representation of the underlying business reality. As we proved in Sec 3.2, this assumption is violated when systematic errors are present---precisely the situation where audit analytics is most needed. DataSynth addresses this gap..." with:

"These approaches generally rely on available operational data as their starting point and do not separately establish that the input faithfully represents the underlying business reality. As discussed in \Cref{sec:gt_errors}, when systematic errors are present---precisely the situation where audit analytics is most critical---this reliance can limit the reliability of the resulting knowledge structures. \system{} complements these approaches by providing reference knowledge graphs with known ground truth, enabling rigorous evaluation of knowledge construction methods."

Lines 54-59: Replace "they treat tables as independent collections of rows and do not enforce cross-table referential integrity..." with: "While these methods excel at preserving marginal distributions and pairwise correlations---a significant achievement for general-purpose tabular synthesis---the enterprise accounting domain imposes additional requirements: cross-table referential integrity, temporal ordering constraints, and domain-specific algebraic invariants (e.g., balanced entries). Furthermore, audit analytics requires a normative knowledge layer that is beyond the scope of general-purpose generators."

Lines 74-77: Replace "None of these systems produce a complete knowledge graph... They generate isolated data artifacts rather than interconnected reference knowledge, limiting their utility..." with: "These systems were designed to address specific aspects of the synthetic data challenge and serve their intended purposes effectively. However, they do not produce a comprehensive reference knowledge graph with provenance across all three layers, which is what evaluating end-to-end knowledge construction pipelines requires."

- [ ] **Step 5: anomaly_injection.tex — calibrate claims**

Lines 8-9: Replace "ground-truth labels that are impossible to obtain from real enterprise data" with "comprehensive ground-truth labels that are difficult to obtain from real enterprise data alone"

Lines 140-143: Replace "they provide perfect ground truth---unlike labels derived from real data, which are inherently incomplete and potentially biased" with "they provide complete and consistent ground truth with respect to the generative model---complementing labels derived from real data, which, while grounded in actual patterns, are typically incomplete and may be influenced by the same systematic errors they seek to identify"

- [ ] **Step 6: architecture.tex — soften provenance claim**

Lines 136-138: Replace "a property that is impossible to achieve with inverse-recovery approaches" with "a property that is difficult to achieve comprehensively when constructing knowledge from observed data alone"

- [ ] **Step 7: knowledge_graph.tex — complement not compete**

Lines 29-32: Replace "Unlike accounting networks constructed from real data via inverse recovery" with "Complementing accounting networks constructed from real data"

- [ ] **Step 8: conclusion.tex — balanced framing**

Lines 7-9: Replace "building knowledge systems by scraping existing enterprise data---the dominant approach---is analogous to solving a jigsaw puzzle without the picture" with "constructing knowledge systems primarily from operational enterprise data---while valuable and widely practiced---faces inherent limitations when the source data itself may contain systematic errors"

Lines 11-13: Replace "the constructed knowledge inherits and amplifies them, treating symptoms while root causes remain undiagnosed" with "the constructed knowledge may inherit those errors, potentially masking the root causes that audit procedures seek to identify"

- [ ] **Step 9: Add complementarity statement to abstract**

After the existing abstract text about what DataSynth does, and before the Keywords, add a sentence:

"The forward-generation approach does not replace the analysis of real enterprise data; rather, it provides a reference frame against which analytical tools, knowledge construction pipelines, and audit procedures can be developed, tested, and calibrated."

(This echoes the good qualifying language already in ground_truth.tex Section 3.5 but makes it visible in the abstract.)

---

### Task 6: Implementation Boasting Removal

**Files:**
- Modify: `paper/sections/abstract.tex`
- Modify: `paper/sections/introduction.tex`
- Modify: `paper/sections/architecture.tex`
- Modify: `paper/sections/conclusion.tex`

- [ ] **Step 1: abstract.tex — remove LoC and crate count**

Line 45: Replace "Implemented in Rust across 16 crates totaling over 120,k lines of code, \system{} achieves throughput exceeding..." with "Implemented in Rust, \system{} achieves throughput exceeding..."

Line 52: Replace "a 33-type anomaly injection framework" with "a multi-type anomaly injection framework"

- [ ] **Step 2: introduction.tex — remove crate count**

Line 102: Replace "comprising 16 Rust crates that separate" with "that separates"

- [ ] **Step 3: architecture.tex — reduce counting**

Line 49: Replace "defines over 200 domain model types spanning" with "defines domain model types spanning"

Line 75: Replace "contains 50+ specialized generators" with "contains specialized generators"

- [ ] **Step 4: Deduplicate scale numbers across paper**

Keep "78+ entity types and 39+ relationship types" ONLY in knowledge_graph.tex (Section 6, where it's substantive). Remove from abstract, introduction, and conclusion.

Keep "20+ enterprise processes" ONLY in generation_pipeline.tex. Remove from abstract and conclusion.

Keep "15+ coherence validators" ONLY in evaluation.tex. Remove elsewhere.

- [ ] **Step 5: conclusion.tex — trim repeated claims**

Lines 30-42: Reduce to essentials. The conclusion should summarize the scientific contribution, not repeat throughput and feature counts. Remove the "over $2 \times 10^5$ journal entries per second" from the conclusion (it's in the experiments section). Remove "covers 20+ enterprise processes" (it's in the pipeline section). Keep the key results (100% invariant compliance, Benford MAD, F1 scores) since those are scientific findings.

---

### Task 7: Comparison Table Fix + Final Consistency Pass

**Files:**
- Modify: `paper/sections/related_work.tex`
- Modify: all .tex files (consistency check)

- [ ] **Step 1: Update comparison table in related_work.tex**

Remove the SAFD row entirely (reference doesn't exist).

Fix the GL.ai row: Either remove it (since GL.ai is not a data generation system) or replace with "Schreyer et al." and adjust the checkmarks to reflect what their federated learning framework actually does (which is none of the listed features — it's a learning method, not a generator). Recommendation: remove the row to avoid misrepresentation.

Verify remaining rows (CTGAN/TVAE, SDV/CopulaGAN, REaLTabFormer, BankSim, Guo et al.) are accurate.

- [ ] **Step 2: Update all cross-references to renamed theorems**

After downgrading Theorem 3.4 to Observation and Theorem 3.6 to Proposition, search all .tex files for `\Cref{thm:amplification}` and `\Cref{thm:completeness}` and verify the references still resolve correctly (they should, since the labels don't change, only the environment names).

- [ ] **Step 3: Search for any remaining old cite keys**

Grep all .tex files for: `lopez2018`, `arya2010`, `dbouk2023`, `sharma2023` to ensure no stale references remain.

- [ ] **Step 4: Check that the .bbl file note**

Add a comment at the top of references.bib noting that the .bbl file needs regeneration after these changes:
```
% NOTE: After editing, recompile with: pdflatex main && bibtex main && pdflatex main && pdflatex main
```

---

## Execution Notes

- Tasks 1-2 (references) should be done first since they affect what the comparison table and related work text can say.
- Task 3 (formulas) and Task 4 (theorems) can be done in parallel.
- Task 5 (tone) should be done after Tasks 3-4 since theorem restructuring changes some text.
- Task 6 (boasting) can be done in parallel with Task 5.
- Task 7 (consistency) must be last.
