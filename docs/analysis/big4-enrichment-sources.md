# Big 4 Blueprint Enrichment Sources

Documentation of public sources used to enrich the firm-specific audit methodology blueprints beyond the ISA-37 base.

## Enrichment Principles

1. All enrichments are based on **publicly available information** only
2. Sources include: firm press releases, PCAOB inspection reports (public government documents), partner marketing materials, transparency reports, academic papers, and industry publications
3. Tool names are **publicly marketed brand names** used in the firms' own publications
4. No proprietary methodology content was accessed or reproduced
5. Blueprints carry the disclaimer: *"Based on public ISA standards and publicly documented firm capabilities, not proprietary firm methodologies."*

## KPMG Clara

### New Procedures Added

| Procedure | Source | Public Evidence |
|-----------|--------|----------------|
| `dynamic_risk_assessment_4d` | KPMG DRA brochure (PDF), patent documentation | 4D framework (Likelihood, Impact, Velocity, Connectivity) with graph theory contagion analysis. Published in multiple KPMG regional brochures. |
| `ai_agent_expense_vouching` | KPMG press release (April 2025), Accounting Today | Named AI agent announced publicly. Automates expense-to-documentation matching. |
| `ai_agent_unrecorded_liabilities` | KPMG press release (April 2025), Boardroom Insight | Named AI agent for completeness testing of liabilities. |
| `audit_chat_genai_consultation` | KPMG newsroom (May 2024), Microsoft customer story | 600K+ conversations documented. Built on Azure OpenAI. Gap analysis and contract document analysis. |
| `trusted_ai_governance_gate` | KPMG Trusted AI Framework whitepaper | 10-principle governance framework published publicly. |

### Enriched Procedures

| Procedure | Enhancement | Source |
|-----------|------------|--------|
| `full_population_transaction_scoring` | Added 6 sub-steps for MindBridge 32 algorithms: Benford, rare combinations, temporal, clustering, digit analysis, composite scoring | MindBridge public documentation, KPMG-MindBridge partnership page |
| `sentinel_independence_check` | Added cross-firm global screening, NAS pre-approval, related party extension | KPMG Transparency Report 2024, System of Audit Quality Controls PDF |

### Key Sources
- KPMG DRA Brochure: assets.kpmg.com/content/dam/kpmg/xx/pdf/2017/03/dynamic-risk-assessment-for-audit-brochure.pdf
- KPMG AI Agents (April 2025): kpmg.com/xx/en/media/press-releases/2025/04/kpmg-advances-ai-integration-in-kpmg-clara-smart-audit-platform.html
- KPMG Audit Chat: kpmg.com/us/en/media/news/kpmg-audit-chat-new-capabilities-2024.html
- KPMG + Microsoft: microsoft.com/en/customers/story/25353-kpmg-international-azure
- MindBridge Platform: mindbridge.ai/platform/
- MindBridge 32 Algorithms: mindbridge.ai/blog/anomaly-detection-techniques-how-to-uncover-risks-identify-patterns-and-strengthen-data-integrity/
- KPMG Trusted AI: kpmg.com/xx/en/what-we-do/services/ai/trusted-ai-framework.html
- PCAOB KPMG 2024 Inspection: assets.pcaobus.org/pcaob-dev/docs/default-source/inspections/documents/104-2025-039-kpmg.pdf

## PwC Aura

### New Procedures Added

| Procedure | Source | Public Evidence |
|-----------|--------|----------------|
| `gl_ai_anomaly_detection` | PwC press release, Emerj research, Medium analysis | DBSCAN clustering algorithm (H2O.ai partnership). Multi-factor anomaly scoring. "Audit Innovation of the Year" 2017. |
| `cash_ai_bank_reconciliation` | PwC press release, award announcement | BERT NLP for bank statement parsing (98% accuracy). "Audit Innovation of the Year" 2019. |
| `evidence_match_automated_testing` | Accounting Today (2026), PwC newsroom | Agent-led AR/AP/Cash evidence matching. Announced as part of end-to-end AI automation roadmap. |
| `chatnational_research` | PwC newsroom | GenAI research tool on Azure OpenAI GPT-4 with Viewpoint RAG. 17,000 assurance professionals in first year. |
| `extract_data_ingestion` | PwC regional sites (Middle East, Switzerland, Isle of Man) | Secure multi-system data extraction. Fourth pillar of PwC audit tech ecosystem. |
| `halo_cryptocurrency_assurance` | PwC press release, CoinDesk | Blockchain ownership verification. Supports Bitcoin, Ethereum, Litecoin, XRP. |
| `advanced_walkthrough_assistant` | Accounting Today (2026) | Controls walkthrough automation. Ingests prior year walkthroughs, generates tailored work plans. |

### Enriched Procedures

| Procedure | Enhancement | Source |
|-----------|------------|--------|
| `halo_journal_pattern_analysis` | Added dashboard generation, pattern recognition, temporal anomaly detection, drill-down investigation | PwC Switzerland: 1,500 algorithms, 40+ dashboards. PwC Mauritius Halo documentation. |
| `frisk_13_factor_assessment` | Added disclaimer that FRISK is not a confirmed PwC tool name; aligned with ISA 220/ISQM 1 | Research finding: no public PwC source for "FRISK" name |

### Key Sources
- PwC GL.ai: pwc.com/gx/en/about/stories-from-across-the-world/harnessing-the-power-of-ai-to-transform-the-detection-of-fraud-and-error.html
- PwC Cash.ai Award: pwc.com/sk/en/current-press-releases/cashai-was-named-audit-innovation-of-the-yea-2019.html
- PwC Evidence Match: accountingtoday.com/news/pwc-expects-end-to-end-ai-audit-automation-within-2026
- PwC ChatNational: pwc.com/us/en/about-us/newsroom/press-releases/chatbot-chatnational-anniversary.html
- PwC Halo: linkedin.com/pulse/innovation-halo-effect-maria-castañón-moats
- PwC Switzerland 1,500 algos: pwc.ch/en/services/assurance/technology-enabled-audit.html
- PwC Halo Crypto: pwc.com/gx/en/services/audit-assurance/halo-solution-for-cryptocurrency.html
- PCAOB PwC 2024 Inspection: assets.pcaobus.org/pcaob-dev/docs/default-source/inspections/documents/104-2025-040-pwc.pdf

## Deloitte Omnia

### New Procedures Added

| Procedure | Source | Public Evidence |
|-----------|--------|----------------|
| `cortex_cdm_mapping` | Deloitte press releases (2022-2023), transparency reports | Common Data Model normalization from 150+ ERP systems. Pre-built connectors for SAP, Oracle, Workday, NetSuite. |
| `signal_risk_monitoring` | Deloitte public descriptions | External risk data aggregation (news, filings, market data, credit ratings). Industry benchmarking. Early warning system. |
| `argus_contract_extraction` | Deloitte press releases | NLP document classification and contract term extraction. Lease (ASC 842/IFRS 16) and revenue (ASC 606/IFRS 15) analysis. Confidence scoring. |
| `reveal_network_analysis` | Deloitte public descriptions | Entity relationship mapping, circular transaction detection, timeline reconstruction. Used in forensic/investigation contexts and audit support. |
| `trustworthy_ai_governance` | Deloitte Trustworthy AI whitepaper series (2021-2024) | 6-dimension framework: Fair/Impartial, Robust/Reliable, Privacy, Safe/Secure, Responsible, Transparent/Explainable. |

### Enriched Procedures

| Procedure | Enhancement | Source |
|-----------|------------|--------|
| `spotlight_je_testing` | Added 8 sub-steps for 20+ documented JE tests: temporal, threshold, backdated, unusual combinations, user profiling, one-sided, IC patterns, risk tiering | Public Deloitte descriptions, PCAOB inspection findings |
| `iconfirm_digital_confirmations` | Added response tracking, automated matching, alternative procedures | Public iConfirm/Confirmation.com documentation |

### Key Sources
- Deloitte Omnia Launch: Deloitte press releases 2022-2023
- Deloitte-Google Cloud Alliance: Google Cloud customer stories
- Deloitte Cortex/Spotlight: Deloitte Audit & Assurance website
- Deloitte Trustworthy AI: Deloitte AI Institute publications, Trustworthy AI whitepaper series
- Deloitte PCAOB 2024 Inspection: pcaobus.org inspection reports
- Deloitte Global Impact Report: Annual publication

## PCAOB Cross-Cutting Findings

Applied across all blueprints where relevant:

| Finding | Application | Source |
|---------|------------|--------|
| IPE (Information Produced by Entity) testing | All procedures using client data should include IPE validation | PCAOB Staff Audit Practice Alerts, inspection reports |
| Revenue recognition (ASC 606) emphasis | Revenue testing procedures strengthened | #1 deficiency area across all firms |
| ECL/CECL model validation | Credit loss procedures strengthened | #2 deficiency area |
| MRC precision testing | Management review controls testing enhanced | Recurring ICFR finding |
