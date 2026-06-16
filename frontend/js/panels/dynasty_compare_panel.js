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

window.DynastyComparePanel = DynastyComparePanel;
