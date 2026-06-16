class CrossEraComparePanel {
    constructor(containerId, apiBase) {
        this.container = document.getElementById(containerId);
        this.apiBase = apiBase;
        this.canvas = null;
        this._init();
    }

    _init() {
        this.container.innerHTML = `
            <div class="panel-block" style="border-left: 4px solid #3B82F6;">
                <div class="panel-title">
                    <span class="panel-icon">⏱️</span>
                    跨时代精度对比
                    <button id="btn-cross-era" style="margin-left:auto;padding:4px 10px;background:#3B82F6;color:white;border:none;border-radius:5px;font-size:11px;cursor:pointer;">刷新</button>
                </div>
                <div style="padding: 12px;">
                    <div style="height: 220px; background: #fff; border: 1px solid #e5e7eb; border-radius: 6px; position: relative; overflow: hidden;">
                        <canvas id="cross-era-canvas" style="width:100%;height:100%;"></canvas>
                        <div id="cross-era-loading" style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;background:rgba(255,255,255,0.9);color:#666;font-size:13px;">
                            <span style="animation:spin 1s linear infinite;display:inline-block;margin-right:4px;">⏳</span> 加载精度数据...
                        </div>
                    </div>
                    <div id="cross-era-summary" style="margin-top: 10px; padding: 10px; background: #eff6ff; border-radius: 6px; font-size: 12px; color: #1e40af;">
                        点击"刷新"按钮，查看古代漏壶与现代计时器的精度跨越对比
                    </div>
                </div>
            </div>
        `;
        document.getElementById('btn-cross-era').onclick = () => this.load();
        this.canvas = document.getElementById('cross-era-canvas');
        setTimeout(() => this.load(), 500);
    }

    async load() {
        const loading = document.getElementById('cross-era-loading');
        loading.style.display = 'flex';
        try {
            const resp = await fetch(this.apiBase + '/api/cross-era');
            const json = await resp.json();
            if (json.success && json.data) this._render(json.data);
        } catch(e) { console.warn(e); }
        finally { loading.style.display = 'none'; }
    }

    _render(data) {
        const canvas = this.canvas;
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        const ctx = canvas.getContext('2d');
        ctx.scale(dpr, dpr);
        const W = rect.width, H = rect.height;
        ctx.clearRect(0, 0, W, H);

        const allItems = [
            ...data.ancient_devices,
            ...data.modern_devices,
        ].sort((a, b) => a.daily_error_seconds - b.daily_error_seconds);

        const minErr = Math.min(...allItems.map(i => i.daily_error_seconds));
        const maxErr = Math.max(...allItems.map(i => i.daily_error_seconds));
        const padL = 100, padR = 20, padT = 10, padB = 30;
        const plotW = W - padL - padR, plotH = H - padT - padB;

        const logMin = Math.log10(Math.max(minErr, 1e-7));
        const logMax = Math.log10(maxErr * 1.2);

        const xOf = (err) => padL + plotW * (1 - (Math.log10(Math.max(err, 1e-7)) - logMin) / (logMax - logMin));
        const yOf = (i) => padT + plotH * (i + 0.5) / allItems.length;
        const rowH = plotH / allItems.length;

        ctx.strokeStyle = '#e5e7eb';
        ctx.lineWidth = 1;
        for (let exp = Math.ceil(logMin); exp <= Math.floor(logMax); exp++) {
            const x = xOf(Math.pow(10, exp));
            ctx.beginPath(); ctx.moveTo(x, padT); ctx.lineTo(x, padT + plotH); ctx.stroke();
            ctx.fillStyle = '#9ca3af';
            ctx.font = '10px sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText(this._formatTick(Math.pow(10, exp)), x, padT + plotH + 14);
        }

        allItems.forEach((item, idx) => {
            const y = yOf(idx);
            const isAncient = item.era === '古代';
            ctx.fillStyle = idx % 2 === 0 ? '#fafafa' : 'transparent';
            ctx.fillRect(padL, y - rowH/2, plotW, rowH);
            ctx.fillStyle = '#6b7280';
            ctx.font = '10px sans-serif';
            ctx.textAlign = 'right';
            ctx.textBaseline = 'middle';
            const labelParts = item.label.split('·');
            ctx.fillText(labelParts[0] || item.label, padL - 4, y);

            const x = xOf(item.daily_error_seconds);
            const barW = Math.max(2, x - padL);
            const grad = ctx.createLinearGradient(padL, 0, x, 0);
            grad.addColorStop(0, item.color_hex + '88');
            grad.addColorStop(1, item.color_hex);
            ctx.fillStyle = grad;
            ctx.fillRect(padL, y - rowH*0.3, barW, rowH*0.6);
            ctx.strokeStyle = item.color_hex;
            ctx.strokeRect(padL, y - rowH*0.3, barW, rowH*0.6);

            ctx.beginPath();
            ctx.arc(x, y, 4, 0, Math.PI*2);
            ctx.fillStyle = isAncient ? '#B45309' : '#1D4ED8';
            ctx.fill();
            ctx.strokeStyle = '#fff';
            ctx.lineWidth = 1.5;
            ctx.stroke();

            ctx.fillStyle = '#111827';
            ctx.font = 'bold 10px sans-serif';
            ctx.textAlign = 'left';
            ctx.fillText(' ' + this._formatError(item.daily_error_seconds), x + 2, y);
        });

        const summary = document.getElementById('cross-era-summary');
        const bestA = data.best_ancient, bestM = data.best_modern;
        summary.innerHTML = `
            <div style="font-weight:bold;margin-bottom:4px;">🌍 文明跨越：从汉代到航天时代的计时精度演进</div>
            <div>古代最优：<b style="color:${bestA.color_hex}">${bestA.label}</b> 日误差 <b>${this._formatError(bestA.daily_error_seconds)}</b></div>
            <div>现代最优：<b style="color:${bestM.color_hex}">${bestM.label}</b> 日误差 <b>${this._formatError(bestM.daily_error_seconds)}</b></div>
            <div style="margin-top:4px;font-weight:bold;">精度飞跃：<span style="color:#dc2626;font-size:14px;">${this._formatFactor(data.improvement_factor)} 倍</span>，两千多年人类智慧的凝结！</div>
        `;
    }

    _formatTick(v) {
        if (v >= 1) return v + 's';
        if (v >= 1e-3) return (v*1e3).toFixed(0) + 'ms';
        if (v >= 1e-6) return (v*1e6).toFixed(0) + 'μs';
        return (v*1e9).toFixed(0) + 'ns';
    }

    _formatError(sec) {
        if (sec >= 60) return (sec/60).toFixed(1) + '分';
        if (sec >= 1) return sec.toFixed(2) + '秒';
        if (sec >= 1e-3) return (sec*1e3).toFixed(2) + 'ms';
        if (sec >= 1e-6) return (sec*1e6).toFixed(2) + 'μs';
        return (sec*1e9).toFixed(1) + 'ns';
    }

    _formatFactor(f) {
        if (f >= 1e9) return (f/1e9).toFixed(1) + '×10⁹';
        if (f >= 1e6) return (f/1e6).toFixed(1) + '×10⁶';
        if (f >= 1e4) return (f/1e4).toFixed(1) + '万';
        return f.toFixed(0);
    }
}

window.CrossEraComparePanel = CrossEraComparePanel;
