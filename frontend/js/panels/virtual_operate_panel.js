class VirtualOperatePanel {
    constructor(containerId, apiBase, scene3d) {
        this.container = document.getElementById(containerId);
        this.apiBase = apiBase;
        this.scene3d = scene3d;
        this.clepsydraIds = [
            ['KD1','天上壶'], ['KD2','夜漏壶'], ['KD3','平水壶'], ['KD4','万分水']
        ];
        this.currentResult = null;
        this.animIdx = 0;
        this.animTimer = null;
        this._init();
    }

    _init() {
        const configs = this.clepsydraIds;
        this.container.innerHTML = `
            <div class="panel-block" style="border-left: 4px solid #059669;">
                <div class="panel-title">
                    <span class="panel-icon">🎮</span>
                    公众虚拟操作体验
                    <span style="margin-left:8px;font-size:10px;color:#999;font-weight:normal;">调节水位观察计时变化</span>
                </div>
                <div style="padding: 12px;">
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-bottom:10px;">
                        <div>
                            <label style="font-size:11px;color:#666;">选择漏壶</label>
                            <select id="vo-clepsydra" style="width:100%;padding:7px;border:1px solid #e5e7eb;border-radius:6px;font-size:12px;margin-top:2px;">
                                ${configs.map(c=>`<option value="${c[0]}">${c[0]} ${c[1]}</option>`).join('')}
                            </select>
                        </div>
                        <div>
                            <label style="font-size:11px;color:#666;">水温(°C)</label>
                            <input id="vo-temp" type="number" value="20" min="0" max="50" step="1"
                                style="width:100%;padding:7px;border:1px solid #e5e7eb;border-radius:6px;font-size:12px;margin-top:2px;" />
                        </div>
                    </div>
                    <div style="margin-bottom:8px;">
                        <div style="display:flex;justify-content:space-between;font-size:11px;color:#666;margin-bottom:2px;">
                            <span>目标水位</span>
                            <span id="vo-level-val">70.0 cm</span>
                        </div>
                        <input id="vo-level" type="range" min="5" max="120" value="70" step="0.1"
                            style="width:100%;accent-color:#059669;" />
                        <div style="display:flex;justify-content:space-between;gap:4px;margin-top:6px;">
                            ${[
                                ['25%','1/4水位'],['50%','1/2水位'],
                                ['75%','3/4水位'],['100%','满水位']
                            ].map(([v,l])=>`<button data-level="${v}" class="vo-quick-btn"
                                style="flex:1;padding:4px;border:1px solid #a7f3d0;background:#ecfdf5;color:#065f46;border-radius:5px;font-size:10px;cursor:pointer;">${l}</button>`).join('')}
                        </div>
                    </div>
                    <div style="margin-bottom:10px;">
                        <label style="font-size:11px;color:#666;">模拟时长</label>
                        <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:4px;margin-top:4px;">
                            ${[
                                [60,'1分钟'],[600,'10分钟'],[3600,'1小时'],[86400,'1日']
                            ].map(([v,l])=>`<button data-val="${v}" class="vo-time-btn"
                                style="padding:6px;border:1px solid #d1d5db;background:#f9fafb;border-radius:5px;font-size:11px;cursor:pointer;">${l}</button>`).join('')}
                        </div>
                    </div>
                    <button id="btn-vo-run" style="width:100%;padding:10px;background:linear-gradient(135deg,#10B981,#059669);color:white;border:none;border-radius:8px;font-weight:bold;cursor:pointer;font-size:13px;">
                        ▶ 开始虚拟实验
                    </button>
                    <div id="vo-status" style="margin-top:10px;text-align:center;color:#059669;font-size:12px;"></div>
                    <div style="height:180px;margin-top:10px;border:1px solid #e5e7eb;border-radius:6px;background:#fff;overflow:hidden;">
                        <canvas id="vo-canvas" style="width:100%;height:100%;"></canvas>
                    </div>
                    <div id="vo-observe" style="margin-top:10px;padding:10px;background:#ecfdf5;border:1px solid #a7f3d0;border-radius:6px;font-size:12px;color:#065f46;">
                        点击"开始虚拟实验"，通过调节水位观察流量和计时误差的动态变化
                    </div>
                </div>
            </div>
        `;

        document.querySelectorAll('.vo-time-btn').forEach(b => {
            b.onclick = (e) => {
                document.querySelectorAll('.vo-time-btn').forEach(x => {
                    x.style.background = '#f9fafb'; x.style.color = '#111'; x.style.borderColor = '#d1d5db';
                });
                e.target.style.background = '#059669'; e.target.style.color = '#fff'; e.target.style.borderColor = '#059669';
                this._simSeconds = parseInt(e.target.dataset.val);
            };
        });
        document.querySelector('.vo-time-btn').click();

        const levelSlider = document.getElementById('vo-level');
        const levelVal = document.getElementById('vo-level-val');
        const self = this;
        this._levelDebounceTimer = null;

        const updateScene3d = () => {
            if (!this.scene3d || !this.scene3d.clepsydraScene) return;
            const cid = document.getElementById('vo-clepsydra').value;
            const cfg = this._clepConfig(cid);
            if (!cfg) return;
            const ratio = (parseFloat(levelSlider.value) - cfg.min_level) / (cfg.max_level - cfg.min_level);
            this.scene3d.clepsydraScene.updateWaterLevel(cid, Math.max(0, Math.min(1, ratio)));
        };

        levelSlider.oninput = () => {
            levelVal.textContent = parseFloat(levelSlider.value).toFixed(1) + ' cm';
            if (this._levelDebounceTimer) clearTimeout(this._levelDebounceTimer);
            this._levelDebounceTimer = setTimeout(() => {
                this._levelDebounceTimer = null;
                updateScene3d.call(self);
            }, 16);
        };

        document.querySelectorAll('.vo-quick-btn').forEach(b => {
            b.onclick = () => {
                const pct = parseFloat(b.dataset.level) / 100;
                const min = parseFloat(levelSlider.min);
                const max = parseFloat(levelSlider.max);
                levelSlider.value = (min + (max - min) * pct).toFixed(1);
                levelVal.textContent = parseFloat(levelSlider.value).toFixed(1) + ' cm';
                updateScene3d.call(self);
            };
        });

        document.getElementById('btn-vo-run').onclick = () => this._runExperiment();
    }

    _clepConfig(id) {
        const defaults = {
            KD1: { min_level: 20, max_level: 120 },
            KD2: { min_level: 15, max_level: 100 },
            KD3: { min_level: 10, max_level: 80 },
            KD4: { min_level: 5, max_level: 60 },
        };
        return defaults[id] || { min_level: 10, max_level: 100 };
    }

    async _runExperiment() {
        if (this.animTimer) { clearInterval(this.animTimer); this.animTimer = null; }
        const cid = document.getElementById('vo-clepsydra').value;
        const level = parseFloat(document.getElementById('vo-level').value);
        const temp = parseFloat(document.getElementById('vo-temp').value);
        const secs = this._simSeconds;
        if (!secs) { alert('请选择模拟时长'); return; }

        const statusDiv = document.getElementById('vo-status');
        statusDiv.innerHTML = '<span style="animation:spin 1s linear infinite;display:inline-block;">⏳</span> 计算中...';

        try {
            const resp = await fetch(this.apiBase + '/api/virtual-operate', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({
                    clepsydra_id: cid, target_water_level_cm: level,
                    water_temp_c: temp, simulate_seconds: secs,
                })
            });
            const json = await resp.json();
            if (json.success && json.data) {
                this.currentResult = json.data;
                this.animIdx = 0;
                this._animateResult();
                if (json.data.observations && json.data.observations.length) {
                    document.getElementById('vo-observe').innerHTML = `
                        <div style="font-weight:bold;margin-bottom:4px;">📝 实验观察结论</div>
                        <ul style="margin:0;padding-left:16px;line-height:1.7;">
                            ${json.data.observations.map(o=>`<li>${o}</li>`).join('')}
                        </ul>
                    `;
                }
            }
        } catch(e) {
            statusDiv.innerHTML = `<span style="color:#dc2626;">错误: ${e.message}</span>`;
        }
    }

    _animateResult() {
        const r = this.currentResult;
        if (!r) return;
        const canvas = document.getElementById('vo-canvas');
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        const ctx = canvas.getContext('2d');
        ctx.scale(dpr, dpr);
        const W = rect.width, H = rect.height;
        const totalPoints = r.level_history.length;

        const drawStep = () => {
            this.animIdx = Math.min(this.animIdx + Math.max(1, Math.floor(totalPoints/100)), totalPoints - 1);
            ctx.clearRect(0, 0, W, H);
            this._drawChart(ctx, W, H, r, this.animIdx);

            const cid = r.clepsydra_id;
            const cfg = this._clepConfig(cid);
            const lv = r.level_history[this.animIdx][1];
            if (this.scene3d && this.scene3d.clepsydraScene && cfg) {
                const ratio = (lv - cfg.min_level) / (cfg.max_level - cfg.min_level);
                this.scene3d.clepsydraScene.updateWaterLevel(cid, Math.max(0, Math.min(1, ratio)));
            }

            const t = r.level_history[this.animIdx][0];
            const err = r.error_history[this.animIdx][1];
            document.getElementById('vo-status').innerHTML =
                `⏱ 进度 ${(t/this._simSeconds*100).toFixed(0)}% | 水位: ${lv.toFixed(1)}cm | 累计误差: ${err.toFixed(2)}秒`;

            if (this.animIdx >= totalPoints - 1) {
                clearInterval(this.animTimer);
                this.animTimer = null;
                document.getElementById('vo-status').innerHTML = `✅ 实验完成 | 总误差 ${err.toFixed(2)}秒`;
            }
        };

        this.animTimer = setInterval(drawStep, 30);
        drawStep();
    }

    _drawChart(ctx, W, H, r, maxIdx) {
        const pad = { l: 50, r: 40, t: 20, b: 28 };
        const pw = W - pad.l - pad.r, ph = H - pad.t - pad.b;
        const lh = r.level_history.slice(0, maxIdx + 1);
        const fh = r.flow_history.slice(0, maxIdx + 1);
        const eh = r.error_history.slice(0, maxIdx + 1);
        if (!lh.length) return;

        const maxT = r.time_elapsed_simulated;
        const lvs = lh.map(p => p[1]);
        const minL = Math.min(...lvs) * 0.95;
        const maxL = Math.max(...lvs) * 1.05;
        const errs = eh.map(p => p[1]);
        const minE = Math.min(...errs, 0) - 0.1;
        const maxE = Math.max(...errs, 0) + 0.1;

        ctx.strokeStyle = '#f3f4f6';
        ctx.lineWidth = 1;
        for (let i = 0; i <= 4; i++) {
            const y = pad.t + ph * i / 4;
            ctx.beginPath(); ctx.moveTo(pad.l, y); ctx.lineTo(pad.l + pw, y); ctx.stroke();
        }

        const drawLine = (arr, minVal, maxVal, color) => {
            ctx.strokeStyle = color;
            ctx.lineWidth = 2;
            ctx.beginPath();
            arr.forEach((p, i) => {
                const x = pad.l + pw * (p[0] / maxT);
                const range = maxVal - minVal;
                const norm = range > 1e-9 ? (p[1] - minVal) / range : 0.5;
                const y = pad.t + ph * (1 - norm);
                if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
            });
            ctx.stroke();
        };

        drawLine(lh, minL, maxL, '#059669');
        drawLine(eh, minE, maxE, '#dc2626');

        ctx.fillStyle = '#059669'; ctx.font = 'bold 10px sans-serif'; ctx.textAlign = 'right';
        ctx.fillText(`水位 ${lh[lh.length-1][1].toFixed(1)}cm`, pad.l - 4, pad.t + 10);
        ctx.fillStyle = '#dc2626';
        ctx.fillText(`误差 ${eh[eh.length-1][1].toFixed(2)}s`, pad.l - 4, pad.t + 24);

        ctx.fillStyle = '#9ca3af'; ctx.font = '9px sans-serif'; ctx.textAlign = 'center';
        const tStep = maxT > 3600 ? 3600 : (maxT > 600 ? 600 : 60);
        for (let t = 0; t <= maxT; t += tStep) {
            const x = pad.l + pw * (t / maxT);
            ctx.fillText(this._fmtT(t), x, pad.t + ph + 14);
        }

        const legend = [['#059669','水位(cm)'],['#dc2626','误差(秒)']];
        legend.forEach((l, i) => {
            const x = pad.l + pw - 90 + i*70;
            ctx.fillStyle = l[0];
            ctx.fillRect(x, 4, 10, 3);
            ctx.fillStyle = '#374151';
            ctx.textAlign = 'left';
            ctx.fillText(l[1], x + 14, 8);
        });
    }

    _fmtT(s) {
        if (s >= 86400) return (s/86400).toFixed(0)+'d';
        if (s >= 3600) return (s/3600).toFixed(1)+'h';
        if (s >= 60) return (s/60).toFixed(0)+'m';
        return s+'s';
    }
}

window.VirtualOperatePanel = VirtualOperatePanel;
