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

window.ErrorTransferPanel = ErrorTransferPanel;
