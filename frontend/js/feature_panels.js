class DynastyComparePanel {
    constructor(containerId, apiBase) {
        this.container = document.getElementById(containerId);
        this.apiBase = apiBase;
        this.dynasties = [];
        this.leftSelect = null;
        this.rightSelect = null;
        this.resultDiv = null;
        this._init();
    }

    async _init() {
        this.container.innerHTML = `
            <div class="panel-block" style="border-left: 4px solid #DAA520;">
                <div class="panel-title">
                    <span class="panel-icon">⚔️</span>
                    朝代漏壶精度对比
                </div>
                <div style="padding: 12px;">
                    <div style="display: grid; grid-template-columns: 1fr 60px 1fr; gap: 8px; align-items: center; margin-bottom: 12px;">
                        <select id="dynasty-left" style="width:100%;padding:8px;border:1px solid #e5e7eb;border-radius:6px;background:#fafafa;font-size:13px;">
                            <option value="">选择左侧朝代</option>
                        </select>
                        <div style="text-align:center;font-weight:bold;color:#9ca3af;font-size:18px;">VS</div>
                        <select id="dynasty-right" style="width:100%;padding:8px;border:1px solid #e5e7eb;border-radius:6px;background:#fafafa;font-size:13px;">
                            <option value="">选择右侧朝代</option>
                        </select>
                    </div>
                    <button id="btn-compare-dynasty" style="width:100%;padding:10px;background:linear-gradient(135deg,#DAA520,#B8860B);color:white;border:none;border-radius:8px;font-weight:bold;cursor:pointer;font-size:13px;">
                        开始对比分析
                    </button>
                    <div id="dynasty-compare-result" style="margin-top:12px;"></div>
                </div>
            </div>
        `;
        this.leftSelect = document.getElementById('dynasty-left');
        this.rightSelect = document.getElementById('dynasty-right');
        this.resultDiv = document.getElementById('dynasty-compare-result');
        document.getElementById('btn-compare-dynasty').onclick = () => this._doCompare();
        await this._loadDynasties();
    }

    async _loadDynasties() {
        try {
            const resp = await fetch(this.apiBase + '/api/dynasties');
            const json = await resp.json();
            if (json.success && json.data) {
                this.dynasties = json.data;
                this.dynasties.forEach(d => {
                    const label = `${d.dynasty_name}·${d.era}·${d.clepsydra_type.split('（')[0]} (${d.stage_count}级)`;
                    this.leftSelect.innerHTML += `<option value="${d.dynasty_id}">${label}</option>`;
                    this.rightSelect.innerHTML += `<option value="${d.dynasty_id}">${label}</option>`;
                });
            }
        } catch(e) { console.warn('加载朝代列表失败', e); }
    }

    async _doCompare() {
        const lid = this.leftSelect.value;
        const rid = this.rightSelect.value;
        if (!lid || !rid) { alert('请选择两个朝代'); return; }
        if (lid === rid) { alert('请选择不同的朝代进行对比'); return; }
        try {
            this.resultDiv.innerHTML = '<div style="text-align:center;color:#666;padding:20px;"><span style="animation:spin 1s linear infinite;display:inline-block;">⏳</span> 正在分析...</div>';
            const resp = await fetch(this.apiBase + `/api/dynasties/compare/${lid}/${rid}`);
            const json = await resp.json();
            if (json.success && json.data) this._renderResult(json.data);
            else this.resultDiv.innerHTML = `<div style="color:#ef4444;padding:10px;">对比失败: ${json.message || '未知错误'}</div>`;
        } catch(e) {
            this.resultDiv.innerHTML = `<div style="color:#ef4444;padding:10px;">网络错误: ${e.message}</div>`;
        }
    }

    _renderResult(cmp) {
        const l = cmp.left_dynasty, r = cmp.right_dynasty;
        const le = cmp.left_daily_error_seconds, re = cmp.right_daily_error_seconds;
        const leText = this._formatError(le), reText = this._formatError(re);
        const lWin = le < re;
        let flowTable = '';
        if (cmp.flow_comparison && cmp.flow_comparison.length) {
            flowTable = `
                <div style="margin-top:10px;">
                    <div style="font-size:12px;color:#666;margin-bottom:4px;">各级流量对比（mL/s）</div>
                    <table style="width:100%;font-size:12px;border-collapse:collapse;">
                        <tr style="background:#f3f4f6;">
                            <th style="padding:4px;text-align:left;border:1px solid #e5e7eb;">级别</th>
                            <th style="padding:4px;text-align:center;border:1px solid #e5e7eb;">${l.dynasty_name}</th>
                            <th style="padding:4px;text-align:center;border:1px solid #e5e7eb;">${r.dynasty_name}</th>
                        </tr>
                        ${cmp.flow_comparison.map(f => `
                            <tr>
                                <td style="padding:4px;border:1px solid #e5e7eb;">${f.stage}</td>
                                <td style="padding:4px;text-align:center;border:1px solid #e5e7eb;">${f.left_flow_mlps.toFixed(3)}<br/><span style="font-size:10px;color:#999;">水位${f.left_level_cm.toFixed(0)}cm</span></td>
                                <td style="padding:4px;text-align:center;border:1px solid #e5e7eb;">${f.right_flow_mlps.toFixed(3)}<br/><span style="font-size:10px;color:#999;">水位${f.right_level_cm.toFixed(0)}cm</span></td>
                            </tr>
                        `).join('')}
                    </table>
                </div>
            `;
        }
        this.resultDiv.innerHTML = `
            <div style="border:1px solid #e5e7eb;border-radius:8px;overflow:hidden;">
                <div style="display:grid;grid-template-columns:1fr auto 1fr;gap:8px;padding:12px;background:linear-gradient(135deg,#fef9e7,#fdf2d5);">
                    <div style="text-align:center;padding:8px;background:${lWin?'#d1fae5':'#fee2e2'};border-radius:6px;">
                        <div style="font-weight:bold;font-size:13px;">${l.dynasty_name}</div>
                        <div style="font-size:11px;color:#666;">${l.clepsydra_type}</div>
                        <div style="font-size:20px;font-weight:bold;margin-top:4px;color:${lWin?'#059669':'#dc2626'};">${leText}</div>
                        <div style="font-size:10px;color:#666;">日误差</div>
                        ${lWin?'<div style="font-size:10px;color:#059669;margin-top:2px;">🏆 胜</div>':''}
                    </div>
                    <div style="display:flex;align-items:center;font-weight:bold;color:#666;font-size:24px;">
                        ${le<re ? (re/le).toFixed(1)+'×' : (le/re).toFixed(1)+'×'}
                    </div>
                    <div style="text-align:center;padding:8px;background:${!lWin?'#d1fae5':'#fee2e2'};border-radius:6px;">
                        <div style="font-weight:bold;font-size:13px;">${r.dynasty_name}</div>
                        <div style="font-size:11px;color:#666;">${r.clepsydra_type}</div>
                        <div style="font-size:20px;font-weight:bold;margin-top:4px;color:${!lWin?'#059669':'#dc2626'};">${reText}</div>
                        <div style="font-size:10px;color:#666;">日误差</div>
                        ${!lWin?'<div style="font-size:10px;color:#059669;margin-top:2px;">🏆 胜</div>':''}
                    </div>
                </div>
                <div style="padding:12px;border-top:1px solid #e5e7eb;">
                    <div style="font-size:12px;font-weight:bold;color:#374151;margin-bottom:6px;">🔑 关键差异</div>
                    <ul style="margin:0;padding-left:18px;font-size:12px;color:#4b5563;line-height:1.8;">
                        ${cmp.key_differences.map(d=>`<li>${d}</li>`).join('')}
                    </ul>
                    ${flowTable}
                </div>
            </div>
        `;
    }

    _formatError(sec) {
        if (sec < 60) return sec.toFixed(1) + '秒';
        if (sec < 3600) return (sec/60).toFixed(1) + '分';
        return (sec/3600).toFixed(1) + '时';
    }
}

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

class ErrorTransferPanel {
    constructor(containerId, apiBase) {
        this.container = document.getElementById(containerId);
        this.apiBase = apiBase;
        this.select = null;
        this.resultDiv = null;
        this.canvas = null;
        this._init();
    }

    _init() {
        const dynasties = [
            ['HAN_CHENJIAN','汉·沉箭漏 (1级)'],
            ['HAN_FUJIAN','汉·浮箭漏 (2级)'],
            ['TANG_JINGLU','唐·吕才漏 (4级)'],
            ['SONG_LIANHUA','宋·莲花漏 (3级)'],
            ['SONG_YITIAN','宋·水运仪象台 (4级)'],
            ['YONG_LE','明·永乐漏 (4级)'],
        ];
        this.container.innerHTML = `
            <div class="panel-block" style="border-left: 4px solid #8B5CF6;">
                <div class="panel-title">
                    <span class="panel-icon">🔗</span>
                    多级漏壶误差传递分析
                </div>
                <div style="padding: 12px;">
                    <div style="margin-bottom:10px;">
                        <select id="et-dynasty" style="width:100%;padding:8px;border:1px solid #e5e7eb;border-radius:6px;background:#fafafa;font-size:13px;">
                            ${dynasties.map(d=>`<option value="${d[0]}">${d[1]}</option>`).join('')}
                        </select>
                    </div>
                    <button id="btn-analyze-et" style="width:100%;padding:10px;background:linear-gradient(135deg,#8B5CF6,#7C3AED);color:white;border:none;border-radius:8px;font-weight:bold;cursor:pointer;font-size:13px;">
                        分析误差传递链路
                    </button>
                    <div style="height:240px;margin-top:12px;border:1px solid #e5e7eb;border-radius:6px;background:#fff;overflow:hidden;position:relative;">
                        <canvas id="et-canvas" style="width:100%;height:100%;"></canvas>
                        <div id="et-loading" style="position:absolute;inset:0;display:none;align-items:center;justify-content:center;background:rgba(255,255,255,0.9);color:#666;font-size:13px;">
                            <span style="animation:spin 1s linear infinite;margin-right:4px;">⏳</span> 分析中...
                        </div>
                    </div>
                    <div id="et-result" style="margin-top:10px;"></div>
                </div>
            </div>
        `;
        this.select = document.getElementById('et-dynasty');
        this.resultDiv = document.getElementById('et-result');
        this.canvas = document.getElementById('et-canvas');
        document.getElementById('btn-analyze-et').onclick = () => this.analyze();
        setTimeout(() => this.analyze(), 800);
    }

    async analyze() {
        const did = this.select.value;
        const loading = document.getElementById('et-loading');
        loading.style.display = 'flex';
        try {
            const resp = await fetch(this.apiBase + `/api/error-transfer/${did}`);
            const json = await resp.json();
            if (json.success && json.data) {
                this._drawGraph(json.data);
                this._renderResult(json.data);
            }
        } catch(e) { console.warn(e); }
        finally { loading.style.display = 'none'; }
    }

    _drawGraph(data) {
        const canvas = this.canvas;
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        const ctx = canvas.getContext('2d');
        ctx.scale(dpr, dpr);
        const W = rect.width, H = rect.height;
        ctx.clearRect(0, 0, W, H);

        const nodes = data.nodes;
        if (!nodes.length) return;

        const nodeW = 110, nodeH = 70;
        const gapX = nodes.length > 1 ? (W - 40 - nodeW * nodes.length) / (nodes.length - 1) : 0;
        const centerY = H / 2;

        for (let i = 0; i < nodes.length; i++) {
            const node = nodes[i];
            const x = 20 + i * (nodeW + gapX);
            const y = centerY - nodeH/2;
            const isBn = i === data.bottleneck_stage;

            if (i < nodes.length - 1) {
                const x2 = 20 + (i+1) * (nodeW + gapX);
                ctx.strokeStyle = isBn ? '#dc2626' : '#a78bfa';
                ctx.lineWidth = 2;
                ctx.setLineDash([6, 4]);
                ctx.beginPath();
                ctx.moveTo(x + nodeW, centerY);
                ctx.lineTo(x2, centerY);
                ctx.stroke();
                ctx.setLineDash([]);

                const midX = (x + nodeW + x2) / 2;
                ctx.fillStyle = isBn ? '#dc2626' : '#7c3aed';
                ctx.font = 'bold 10px sans-serif';
                ctx.textAlign = 'center';
                ctx.fillText(`×${node.amplification_factor.toFixed(2)}`, midX, centerY - 8);

                const arrowX = x2 - 6;
                ctx.fillStyle = isBn ? '#dc2626' : '#a78bfa';
                ctx.beginPath();
                ctx.moveTo(arrowX, centerY);
                ctx.lineTo(arrowX - 6, centerY - 5);
                ctx.lineTo(arrowX - 6, centerY + 5);
                ctx.closePath();
                ctx.fill();
            }

            const grad = ctx.createLinearGradient(x, y, x, y + nodeH);
            if (isBn) {
                grad.addColorStop(0, '#fecaca');
                grad.addColorStop(1, '#fca5a5');
            } else {
                grad.addColorStop(0, '#ddd6fe');
                grad.addColorStop(1, '#c4b5fd');
            }
            ctx.fillStyle = grad;
            ctx.strokeStyle = isBn ? '#dc2626' : '#7c3aed';
            ctx.lineWidth = 2;
            this._roundRect(ctx, x, y, nodeW, nodeH, 8);
            ctx.fill();
            ctx.stroke();

            if (isBn) {
                ctx.fillStyle = '#dc2626';
                ctx.font = 'bold 9px sans-serif';
                ctx.textAlign = 'center';
                ctx.fillText('⚠️ 瓶颈级', x + nodeW/2, y - 4);
            }

            ctx.fillStyle = '#1f2937';
            ctx.font = 'bold 11px sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText(`第${i+1}级: ${node.clepsydra_id}`, x + nodeW/2, y + 16);
            ctx.font = '10px sans-serif';
            ctx.fillStyle = '#374151';
            ctx.fillText(`自身误差: ${node.self_error_seconds.toFixed(2)}s`, x + nodeW/2, y + 32);
            ctx.fillText(`贡献: ${node.contribution_percent.toFixed(1)}%`, x + nodeW/2, y + 46);
            ctx.fillStyle = '#6b7280';
            ctx.font = '9px sans-serif';
            ctx.fillText(`输出: ${node.output_error_seconds.toFixed(2)}s`, x + nodeW/2, y + 60);
        }

        ctx.fillStyle = '#111827';
        ctx.font = 'bold 10px sans-serif';
        ctx.textAlign = 'left';
        ctx.fillText(`总日误差: ${data.total_error_seconds.toFixed(2)}秒`, 8, 16);
        ctx.fillText(`可补偿潜力: -${data.compensation_potential_seconds.toFixed(2)}秒 (约${(data.compensation_potential_seconds*100/data.total_error_seconds).toFixed(0)}%)`, 8, 30);
    }

    _roundRect(ctx, x, y, w, h, r) {
        ctx.beginPath();
        ctx.moveTo(x + r, y);
        ctx.arcTo(x + w, y, x + w, y + h, r);
        ctx.arcTo(x + w, y + h, x, y + h, r);
        ctx.arcTo(x, y + h, x, y, r);
        ctx.arcTo(x, y, x + w, y, r);
        ctx.closePath();
    }

    _renderResult(data) {
        this.resultDiv.innerHTML = `
            <div style="background:#fdf4ff;border:1px solid #e9d5ff;border-radius:6px;padding:10px;">
                <div style="font-weight:bold;color:#6b21a8;margin-bottom:4px;">🔍 诊断结论</div>
                <div style="font-size:12px;color:#4c1d95;margin-bottom:6px;">${data.bottleneck_reason}</div>
                <div style="font-weight:bold;color:#6b21a8;font-size:11px;margin-top:8px;margin-bottom:4px;">💡 优化建议</div>
                <ul style="margin:0;padding-left:16px;font-size:11px;color:#581c87;line-height:1.7;">
                    ${data.recommendations.map(r=>`<li>${r}</li>`).join('')}
                </ul>
            </div>
        `;
    }
}

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
        this._lastSceneUpdate = 0;

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
            // 16ms节流（约1帧）防抖：避免每像素都触发3D重建
            if (this._levelDebounceTimer) clearTimeout(this._levelDebounceTimer);
            this._levelDebounceTimer = setTimeout(() => {
                this._levelDebounceTimer = null;
                updateScene3d.call(self);
            }, 16);
        };

        // 快速跳转预设水位按钮：直接定位 + 立绘更
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
        const fls = fh.map(p => p[1]);
        const minF = Math.min(...fls) * 0.9;
        const maxF = Math.max(...fls) * 1.1;

        ctx.strokeStyle = '#f3f4f6';
        ctx.lineWidth = 1;
        for (let i = 0; i <= 4; i++) {
            const y = pad.t + ph * i / 4;
            ctx.beginPath(); ctx.moveTo(pad.l, y); ctx.lineTo(pad.l + pw, y); ctx.stroke();
        }

        const drawLine = (arr, valKey, minVal, maxVal, color) => {
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

        drawLine(lh, 1, minL, maxL, '#059669');
        drawLine(eh, 1, minE, maxE, '#dc2626');

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
