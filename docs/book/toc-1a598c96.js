// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="index.html">Introduction</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Getting Started</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="getting-started/index.html"><strong aria-hidden="true">1.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="getting-started/installation.html"><strong aria-hidden="true">2.</strong> Installation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="getting-started/quick-start.html"><strong aria-hidden="true">3.</strong> Quick Start</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="getting-started/demo-mode.html"><strong aria-hidden="true">4.</strong> Demo Mode</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">User Guide</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/index.html"><strong aria-hidden="true">5.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/cli-reference.html"><strong aria-hidden="true">6.</strong> CLI Reference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/server-api.html"><strong aria-hidden="true">7.</strong> Server API</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/desktop-ui.html"><strong aria-hidden="true">8.</strong> Desktop UI</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/output-formats.html"><strong aria-hidden="true">9.</strong> Output Formats</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/erp-output-formats.html"><strong aria-hidden="true">10.</strong> ERP Output Formats</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/streaming-output.html"><strong aria-hidden="true">11.</strong> Streaming Output</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/python-wrapper-spec.html"><strong aria-hidden="true">12.</strong> Python Wrapper Specification</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/python-wrapper.html"><strong aria-hidden="true">13.</strong> Python Wrapper Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user-guide/ecosystem-integrations.html"><strong aria-hidden="true">14.</strong> Ecosystem Integrations</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Configuration</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/index.html"><strong aria-hidden="true">15.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/yaml-schema.html"><strong aria-hidden="true">16.</strong> YAML Schema Reference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/industry-presets.html"><strong aria-hidden="true">17.</strong> Industry Presets</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/global-settings.html"><strong aria-hidden="true">18.</strong> Global Settings</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/companies.html"><strong aria-hidden="true">19.</strong> Companies</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/transactions.html"><strong aria-hidden="true">20.</strong> Transactions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/master-data.html"><strong aria-hidden="true">21.</strong> Master Data</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/document-flows.html"><strong aria-hidden="true">22.</strong> Document Flows</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/subledgers.html"><strong aria-hidden="true">23.</strong> Subledgers</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/fx-currency.html"><strong aria-hidden="true">24.</strong> FX &amp; Currency</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/financial-settings.html"><strong aria-hidden="true">25.</strong> Financial Settings</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/compliance.html"><strong aria-hidden="true">26.</strong> Compliance</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/output-settings.html"><strong aria-hidden="true">27.</strong> Output Settings</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="configuration/ai-ml-features.html"><strong aria-hidden="true">28.</strong> AI &amp; ML Features</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Architecture</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/index.html"><strong aria-hidden="true">29.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/workspace-layout.html"><strong aria-hidden="true">30.</strong> Workspace Layout</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/domain-models.html"><strong aria-hidden="true">31.</strong> Domain Models</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/data-flow.html"><strong aria-hidden="true">32.</strong> Data Flow</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/generation-pipeline.html"><strong aria-hidden="true">33.</strong> Generation Pipeline</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/memory-management.html"><strong aria-hidden="true">34.</strong> Memory Management</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/process-chains.html"><strong aria-hidden="true">35.</strong> Process Chains</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/design-decisions.html"><strong aria-hidden="true">36.</strong> Design Decisions</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Crate Reference</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/index.html"><strong aria-hidden="true">37.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-core.html"><strong aria-hidden="true">38.</strong> datasynth-core</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-config.html"><strong aria-hidden="true">39.</strong> datasynth-config</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-generators.html"><strong aria-hidden="true">40.</strong> datasynth-generators</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-output.html"><strong aria-hidden="true">41.</strong> datasynth-output</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-runtime.html"><strong aria-hidden="true">42.</strong> datasynth-runtime</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-graph.html"><strong aria-hidden="true">43.</strong> datasynth-graph</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-cli.html"><strong aria-hidden="true">44.</strong> datasynth-cli</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-server.html"><strong aria-hidden="true">45.</strong> datasynth-server</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-ui.html"><strong aria-hidden="true">46.</strong> datasynth-ui</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-eval.html"><strong aria-hidden="true">47.</strong> datasynth-eval</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-banking.html"><strong aria-hidden="true">48.</strong> datasynth-banking</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-ocpm.html"><strong aria-hidden="true">49.</strong> datasynth-ocpm</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-fingerprint.html"><strong aria-hidden="true">50.</strong> datasynth-fingerprint</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-standards.html"><strong aria-hidden="true">51.</strong> datasynth-standards</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/datasynth-test-utils.html"><strong aria-hidden="true">52.</strong> datasynth-test-utils</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Advanced Topics</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/index.html"><strong aria-hidden="true">53.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/fraud-patterns.html"><strong aria-hidden="true">54.</strong> Fraud Patterns &amp; ACFE Taxonomy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/industry-specific.html"><strong aria-hidden="true">55.</strong> Industry-Specific Features</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/anomaly-injection.html"><strong aria-hidden="true">56.</strong> Anomaly Injection</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/data-quality.html"><strong aria-hidden="true">57.</strong> Data Quality Variations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/graph-export.html"><strong aria-hidden="true">58.</strong> Graph Export</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/intercompany.html"><strong aria-hidden="true">59.</strong> Intercompany Processing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/interconnectivity.html"><strong aria-hidden="true">60.</strong> Interconnectivity &amp; Relationships</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/period-close.html"><strong aria-hidden="true">61.</strong> Period Close Engine</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/fingerprinting.html"><strong aria-hidden="true">62.</strong> Fingerprinting</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/accounting-standards.html"><strong aria-hidden="true">63.</strong> Accounting &amp; Audit Standards</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/performance.html"><strong aria-hidden="true">64.</strong> Performance Tuning</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/llm-generation.html"><strong aria-hidden="true">65.</strong> LLM-Augmented Generation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/diffusion-models.html"><strong aria-hidden="true">66.</strong> Diffusion Models</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/causal-generation.html"><strong aria-hidden="true">67.</strong> Causal &amp; Counterfactual Generation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/federated-fingerprinting.html"><strong aria-hidden="true">68.</strong> Federated Fingerprinting</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/certificates.html"><strong aria-hidden="true">69.</strong> Synthetic Data Certificates</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Deployment &amp; Operations</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/index.html"><strong aria-hidden="true">70.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/docker.html"><strong aria-hidden="true">71.</strong> Docker</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/kubernetes.html"><strong aria-hidden="true">72.</strong> Kubernetes</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/bare-metal.html"><strong aria-hidden="true">73.</strong> Bare Metal</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/runbook.html"><strong aria-hidden="true">74.</strong> Operational Runbook</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/capacity-planning.html"><strong aria-hidden="true">75.</strong> Capacity Planning</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/disaster-recovery.html"><strong aria-hidden="true">76.</strong> Disaster Recovery</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/api-reference.html"><strong aria-hidden="true">77.</strong> API Reference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/security-hardening.html"><strong aria-hidden="true">78.</strong> Security Hardening</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="deployment/tls-reverse-proxy.html"><strong aria-hidden="true">79.</strong> TLS &amp; Reverse Proxy</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Use Cases</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/index.html"><strong aria-hidden="true">80.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/fraud-detection.html"><strong aria-hidden="true">81.</strong> Fraud Detection ML</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/audit-analytics.html"><strong aria-hidden="true">82.</strong> Audit Analytics</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/sox-compliance.html"><strong aria-hidden="true">83.</strong> SOX Compliance Testing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/process-mining.html"><strong aria-hidden="true">84.</strong> Process Mining</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/aml-kyc-testing.html"><strong aria-hidden="true">85.</strong> AML/KYC Testing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/erp-testing.html"><strong aria-hidden="true">86.</strong> ERP Load Testing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/causal-analysis.html"><strong aria-hidden="true">87.</strong> Causal Analysis</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/llm-training-data.html"><strong aria-hidden="true">88.</strong> LLM Training Data</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="use-cases/pipeline-orchestration.html"><strong aria-hidden="true">89.</strong> Pipeline Orchestration</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Contributing</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/index.html"><strong aria-hidden="true">90.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/development-setup.html"><strong aria-hidden="true">91.</strong> Development Setup</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/code-style.html"><strong aria-hidden="true">92.</strong> Code Style Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/testing.html"><strong aria-hidden="true">93.</strong> Testing Guidelines</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/pull-requests.html"><strong aria-hidden="true">94.</strong> Pull Request Process</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Compliance &amp; Regulatory</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="compliance/index.html"><strong aria-hidden="true">95.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="compliance/eu-ai-act.html"><strong aria-hidden="true">96.</strong> EU AI Act</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="compliance/nist-ai-rmf.html"><strong aria-hidden="true">97.</strong> NIST AI RMF</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="compliance/gdpr.html"><strong aria-hidden="true">98.</strong> GDPR</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="compliance/soc2.html"><strong aria-hidden="true">99.</strong> SOC 2 Readiness</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="compliance/iso27001.html"><strong aria-hidden="true">100.</strong> ISO 27001 Alignment</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">Roadmap</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="roadmap/index.html"><strong aria-hidden="true">101.</strong> Enterprise Simulation &amp; ML Ground Truth</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="roadmap/production-readiness.html"><strong aria-hidden="true">102.</strong> Production Readiness</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="roadmap/research/index.html"><strong aria-hidden="true">103.</strong> Research: System Improvements</a><a class="chapter-fold-toggle"><div>❱</div></a></span><ol class="section"><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/01-realism-names-metadata.html"><strong aria-hidden="true">103.1.</strong> Realism: Names &amp; Metadata</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/02-statistical-distributions.html"><strong aria-hidden="true">103.2.</strong> Statistical Distributions</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/03-temporal-patterns.html"><strong aria-hidden="true">103.3.</strong> Temporal Patterns</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/04-interconnectivity.html"><strong aria-hidden="true">103.4.</strong> Interconnectivity</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/05-pattern-drift.html"><strong aria-hidden="true">103.5.</strong> Pattern &amp; Process Drift</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/06-anomaly-patterns.html"><strong aria-hidden="true">103.6.</strong> Anomaly Patterns</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/07-fraud-patterns.html"><strong aria-hidden="true">103.7.</strong> Fraud Patterns</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="roadmap/research/08-domain-specific.html"><strong aria-hidden="true">103.8.</strong> Domain-Specific Enhancements</a></span></li></ol><li class="chapter-item expanded "><li class="spacer"></li></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            if (link.href === current_page
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                const clientRect = e.target.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                sessionStorage.setItem('sidebar-scroll-offset', clientRect.top - sidebarRect.top);
            }
        }, { passive: true });
        const sidebarScrollOffset = sessionStorage.getItem('sidebar-scroll-offset');
        sessionStorage.removeItem('sidebar-scroll-offset');
        if (sidebarScrollOffset !== null) {
            // preserve sidebar scroll position when navigating via links within sidebar
            const activeSection = this.querySelector('.active');
            if (activeSection) {
                const clientRect = activeSection.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                const currentOffset = clientRect.top - sidebarRect.top;
                this.scrollTop += currentOffset - parseFloat(sidebarScrollOffset);
            }
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);


// ---------------------------------------------------------------------------
// Support for dynamically adding headers to the sidebar.

(function() {
    // This is used to detect which direction the page has scrolled since the
    // last scroll event.
    let lastKnownScrollPosition = 0;
    // This is the threshold in px from the top of the screen where it will
    // consider a header the "current" header when scrolling down.
    const defaultDownThreshold = 150;
    // Same as defaultDownThreshold, except when scrolling up.
    const defaultUpThreshold = 300;
    // The threshold is a virtual horizontal line on the screen where it
    // considers the "current" header to be above the line. The threshold is
    // modified dynamically to handle headers that are near the bottom of the
    // screen, and to slightly offset the behavior when scrolling up vs down.
    let threshold = defaultDownThreshold;
    // This is used to disable updates while scrolling. This is needed when
    // clicking the header in the sidebar, which triggers a scroll event. It
    // is somewhat finicky to detect when the scroll has finished, so this
    // uses a relatively dumb system of disabling scroll updates for a short
    // time after the click.
    let disableScroll = false;
    // Array of header elements on the page.
    let headers;
    // Array of li elements that are initially collapsed headers in the sidebar.
    // I'm not sure why eslint seems to have a false positive here.
    // eslint-disable-next-line prefer-const
    let headerToggles = [];
    // This is a debugging tool for the threshold which you can enable in the console.
    let thresholdDebug = false;

    // Updates the threshold based on the scroll position.
    function updateThreshold() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // The number of pixels below the viewport, at most documentHeight.
        // This is used to push the threshold down to the bottom of the page
        // as the user scrolls towards the bottom.
        const pixelsBelow = Math.max(0, documentHeight - (scrollTop + windowHeight));
        // The number of pixels above the viewport, at least defaultDownThreshold.
        // Similar to pixelsBelow, this is used to push the threshold back towards
        // the top when reaching the top of the page.
        const pixelsAbove = Math.max(0, defaultDownThreshold - scrollTop);
        // How much the threshold should be offset once it gets close to the
        // bottom of the page.
        const bottomAdd = Math.max(0, windowHeight - pixelsBelow - defaultDownThreshold);
        let adjustedBottomAdd = bottomAdd;

        // Adjusts bottomAdd for a small document. The calculation above
        // assumes the document is at least twice the windowheight in size. If
        // it is less than that, then bottomAdd needs to be shrunk
        // proportional to the difference in size.
        if (documentHeight < windowHeight * 2) {
            const maxPixelsBelow = documentHeight - windowHeight;
            const t = 1 - pixelsBelow / Math.max(1, maxPixelsBelow);
            const clamp = Math.max(0, Math.min(1, t));
            adjustedBottomAdd *= clamp;
        }

        let scrollingDown = true;
        if (scrollTop < lastKnownScrollPosition) {
            scrollingDown = false;
        }

        if (scrollingDown) {
            // When scrolling down, move the threshold up towards the default
            // downwards threshold position. If near the bottom of the page,
            // adjustedBottomAdd will offset the threshold towards the bottom
            // of the page.
            const amountScrolledDown = scrollTop - lastKnownScrollPosition;
            const adjustedDefault = defaultDownThreshold + adjustedBottomAdd;
            threshold = Math.max(adjustedDefault, threshold - amountScrolledDown);
        } else {
            // When scrolling up, move the threshold down towards the default
            // upwards threshold position. If near the bottom of the page,
            // quickly transition the threshold back up where it normally
            // belongs.
            const amountScrolledUp = lastKnownScrollPosition - scrollTop;
            const adjustedDefault = defaultUpThreshold - pixelsAbove
                + Math.max(0, adjustedBottomAdd - defaultDownThreshold);
            threshold = Math.min(adjustedDefault, threshold + amountScrolledUp);
        }

        if (documentHeight <= windowHeight) {
            threshold = 0;
        }

        if (thresholdDebug) {
            const id = 'mdbook-threshold-debug-data';
            let data = document.getElementById(id);
            if (data === null) {
                data = document.createElement('div');
                data.id = id;
                data.style.cssText = `
                    position: fixed;
                    top: 50px;
                    right: 10px;
                    background-color: 0xeeeeee;
                    z-index: 9999;
                    pointer-events: none;
                `;
                document.body.appendChild(data);
            }
            data.innerHTML = `
                <table>
                  <tr><td>documentHeight</td><td>${documentHeight.toFixed(1)}</td></tr>
                  <tr><td>windowHeight</td><td>${windowHeight.toFixed(1)}</td></tr>
                  <tr><td>scrollTop</td><td>${scrollTop.toFixed(1)}</td></tr>
                  <tr><td>pixelsAbove</td><td>${pixelsAbove.toFixed(1)}</td></tr>
                  <tr><td>pixelsBelow</td><td>${pixelsBelow.toFixed(1)}</td></tr>
                  <tr><td>bottomAdd</td><td>${bottomAdd.toFixed(1)}</td></tr>
                  <tr><td>adjustedBottomAdd</td><td>${adjustedBottomAdd.toFixed(1)}</td></tr>
                  <tr><td>scrollingDown</td><td>${scrollingDown}</td></tr>
                  <tr><td>threshold</td><td>${threshold.toFixed(1)}</td></tr>
                </table>
            `;
            drawDebugLine();
        }

        lastKnownScrollPosition = scrollTop;
    }

    function drawDebugLine() {
        if (!document.body) {
            return;
        }
        const id = 'mdbook-threshold-debug-line';
        const existingLine = document.getElementById(id);
        if (existingLine) {
            existingLine.remove();
        }
        const line = document.createElement('div');
        line.id = id;
        line.style.cssText = `
            position: fixed;
            top: ${threshold}px;
            left: 0;
            width: 100vw;
            height: 2px;
            background-color: red;
            z-index: 9999;
            pointer-events: none;
        `;
        document.body.appendChild(line);
    }

    function mdbookEnableThresholdDebug() {
        thresholdDebug = true;
        updateThreshold();
        drawDebugLine();
    }

    window.mdbookEnableThresholdDebug = mdbookEnableThresholdDebug;

    // Updates which headers in the sidebar should be expanded. If the current
    // header is inside a collapsed group, then it, and all its parents should
    // be expanded.
    function updateHeaderExpanded(currentA) {
        // Add expanded to all header-item li ancestors.
        let current = currentA.parentElement;
        while (current) {
            if (current.tagName === 'LI' && current.classList.contains('header-item')) {
                current.classList.add('expanded');
            }
            current = current.parentElement;
        }
    }

    // Updates which header is marked as the "current" header in the sidebar.
    // This is done with a virtual Y threshold, where headers at or below
    // that line will be considered the current one.
    function updateCurrentHeader() {
        if (!headers || !headers.length) {
            return;
        }

        // Reset the classes, which will be rebuilt below.
        const els = document.getElementsByClassName('current-header');
        for (const el of els) {
            el.classList.remove('current-header');
        }
        for (const toggle of headerToggles) {
            toggle.classList.remove('expanded');
        }

        // Find the last header that is above the threshold.
        let lastHeader = null;
        for (const header of headers) {
            const rect = header.getBoundingClientRect();
            if (rect.top <= threshold) {
                lastHeader = header;
            } else {
                break;
            }
        }
        if (lastHeader === null) {
            lastHeader = headers[0];
            const rect = lastHeader.getBoundingClientRect();
            const windowHeight = window.innerHeight;
            if (rect.top >= windowHeight) {
                return;
            }
        }

        // Get the anchor in the summary.
        const href = '#' + lastHeader.id;
        const a = [...document.querySelectorAll('.header-in-summary')]
            .find(element => element.getAttribute('href') === href);
        if (!a) {
            return;
        }

        a.classList.add('current-header');

        updateHeaderExpanded(a);
    }

    // Updates which header is "current" based on the threshold line.
    function reloadCurrentHeader() {
        if (disableScroll) {
            return;
        }
        updateThreshold();
        updateCurrentHeader();
    }


    // When clicking on a header in the sidebar, this adjusts the threshold so
    // that it is located next to the header. This is so that header becomes
    // "current".
    function headerThresholdClick(event) {
        // See disableScroll description why this is done.
        disableScroll = true;
        setTimeout(() => {
            disableScroll = false;
        }, 100);
        // requestAnimationFrame is used to delay the update of the "current"
        // header until after the scroll is done, and the header is in the new
        // position.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                // Closest is needed because if it has child elements like <code>.
                const a = event.target.closest('a');
                const href = a.getAttribute('href');
                const targetId = href.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    threshold = targetElement.getBoundingClientRect().bottom;
                    updateCurrentHeader();
                }
            });
        });
    }

    // Takes the nodes from the given head and copies them over to the
    // destination, along with some filtering.
    function filterHeader(source, dest) {
        const clone = source.cloneNode(true);
        clone.querySelectorAll('mark').forEach(mark => {
            mark.replaceWith(...mark.childNodes);
        });
        dest.append(...clone.childNodes);
    }

    // Scans page for headers and adds them to the sidebar.
    document.addEventListener('DOMContentLoaded', function() {
        const activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection === null) {
            return;
        }

        const main = document.getElementsByTagName('main')[0];
        headers = Array.from(main.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(h => h.id !== '' && h.children.length && h.children[0].tagName === 'A');

        if (headers.length === 0) {
            return;
        }

        // Build a tree of headers in the sidebar.

        const stack = [];

        const firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (let i = 1; i < firstLevel; i++) {
            const ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) {
                stack[stack.length - 1].ol.appendChild(ol);
            }
            stack.push({level: i + 1, ol: ol});
        }

        // The level where it will start folding deeply nested headers.
        const foldLevel = 3;

        for (let i = 0; i < headers.length; i++) {
            const header = headers[i];
            const level = parseInt(header.tagName.charAt(1));

            const currentLevel = stack[stack.length - 1].level;
            if (level > currentLevel) {
                // Begin nesting to this level.
                for (let nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    const ol = document.createElement('ol');
                    ol.classList.add('section');
                    const last = stack[stack.length - 1];
                    const lastChild = last.ol.lastChild;
                    // Handle the case where jumping more than one nesting
                    // level, which doesn't have a list item to place this new
                    // list inside of.
                    if (lastChild) {
                        lastChild.appendChild(ol);
                    } else {
                        last.ol.appendChild(ol);
                    }
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) {
                    stack.pop();
                }
            }

            const li = document.createElement('li');
            li.classList.add('header-item');
            li.classList.add('expanded');
            if (level < foldLevel) {
                li.classList.add('expanded');
            }
            const span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            const a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            filterHeader(header.children[0], a);
            a.addEventListener('click', headerThresholdClick);
            const nextHeader = headers[i + 1];
            if (nextHeader !== undefined) {
                const nextLevel = parseInt(nextHeader.tagName.charAt(1));
                if (nextLevel > level && level >= foldLevel) {
                    const toggle = document.createElement('a');
                    toggle.classList.add('chapter-fold-toggle');
                    toggle.classList.add('header-toggle');
                    toggle.addEventListener('click', () => {
                        li.classList.toggle('expanded');
                    });
                    const toggleDiv = document.createElement('div');
                    toggleDiv.textContent = '❱';
                    toggle.appendChild(toggleDiv);
                    span.appendChild(toggle);
                    headerToggles.push(li);
                }
            }
            li.appendChild(span);

            const currentParent = stack[stack.length - 1];
            currentParent.ol.appendChild(li);
        }

        const onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        const activeItemSpan = activeSection.parentElement;
        activeItemSpan.after(onThisPage);
    });

    document.addEventListener('DOMContentLoaded', reloadCurrentHeader);
    document.addEventListener('scroll', reloadCurrentHeader, { passive: true });
})();

