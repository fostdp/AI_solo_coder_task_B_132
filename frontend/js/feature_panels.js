// feature_panels.js — 统一入口，加载4个独立Panel组件
// 各组件已拆分为独立文件于 panels/ 目录下

(function() {
    const PANEL_SCRIPTS = [
        'js/panels/dynasty_compare_panel.js',
        'js/panels/cross_era_compare_panel.js',
        'js/panels/error_transfer_panel.js',
        'js/panels/virtual_operate_panel.js',
    ];

    let loaded = 0;
    const total = PANEL_SCRIPTS.length;

    PANEL_SCRIPTS.forEach(src => {
        const s = document.createElement('script');
        s.src = src;
        s.onload = () => {
            loaded++;
            if (loaded === total && typeof window._onFeaturePanelsReady === 'function') {
                window._onFeaturePanelsReady();
            }
        };
        document.head.appendChild(s);
    });

    window.loadFeaturePanels = function(apiBase, scene3d) {
        function tryCreate() {
            if (typeof DynastyComparePanel === 'undefined' ||
                typeof CrossEraComparePanel === 'undefined' ||
                typeof ErrorTransferPanel === 'undefined' ||
                typeof VirtualOperatePanel === 'undefined') {
                setTimeout(tryCreate, 50);
                return;
            }
            new DynastyComparePanel('dynasty-compare-panel', apiBase);
            new CrossEraComparePanel('cross-era-panel', apiBase);
            new ErrorTransferPanel('error-transfer-panel', apiBase);
            new VirtualOperatePanel('virtual-operate-panel', apiBase, scene3d);
        }
        tryCreate();
    };
})();
