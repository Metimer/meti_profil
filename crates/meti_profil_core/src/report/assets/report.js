(function () {
  "use strict";
  var DATA = window.METI_PROFIL;
  if (!DATA) return;

  var SVGNS = "http://www.w3.org/2000/svg";
  function el(tag, attrs, parent) {
    var n = document.createElementNS(SVGNS, tag);
    if (attrs) for (var k in attrs) n.setAttribute(k, attrs[k]);
    if (parent) parent.appendChild(n);
    return n;
  }
  function fmt(x) {
    if (x === null || x === undefined) return "–";
    if (typeof x !== "number") return String(x);
    if (Math.abs(x) >= 1000 || (x !== 0 && Math.abs(x) < 0.01)) return x.toPrecision(4);
    return Math.round(x * 1e4) / 1e4;
  }

  // Shared tooltip.
  var tip = document.createElement("div");
  tip.className = "mp-tooltip";
  document.body.appendChild(tip);
  function showTip(html, e) {
    tip.innerHTML = html;
    tip.style.opacity = "1";
    moveTip(e);
  }
  function moveTip(e) {
    var pad = 12;
    tip.style.left = e.clientX + pad + "px";
    tip.style.top = e.clientY + pad + "px";
  }
  function hideTip() { tip.style.opacity = "0"; }
  function hover(node, html) {
    node.addEventListener("mousemove", function (e) { showTip(html, e); });
    node.addEventListener("mouseleave", hideTip);
  }

  function mapPairs(pairs) {
    var m = {};
    (pairs || []).forEach(function (p) { m[p[0]] = p[1]; });
    return m;
  }
  var NUM = mapPairs(DATA.numeric && DATA.numeric.columns);
  var CAT = mapPairs(DATA.categorical && DATA.categorical.columns);

  function histogram(host, stats) {
    var bins = (stats && stats.histogram) || [];
    if (!bins.length) { host.innerHTML = '<div class="mp-empty">No histogram.</div>'; return; }
    var W = 420, H = 150, pl = 4, pr = 4, pt = 8, pb = 22;
    var iw = W - pl - pr, ih = H - pt - pb;
    var maxC = bins.reduce(function (a, b) { return Math.max(a, b.count); }, 0) || 1;
    var bw = iw / bins.length;
    var svg = el("svg", { viewBox: "0 0 " + W + " " + H }, host);
    el("line", { class: "mp-axis", x1: pl, y1: pt + ih, x2: pl + iw, y2: pt + ih }, svg);
    bins.forEach(function (b, i) {
      var h = (b.count / maxC) * ih;
      var x = pl + i * bw;
      var rect = el("rect", {
        class: "mp-bar", x: x + 1, y: pt + ih - h, width: Math.max(bw - 2, 1), height: h, rx: 1
      }, svg);
      hover(rect, "<b>[" + fmt(b.lower) + ", " + fmt(b.upper) + ")</b><br>count: " + b.count);
    });
    el("text", { class: "mp-axis-text", x: pl, y: H - 6 }, svg).textContent = fmt(bins[0].lower);
    var last = el("text", { class: "mp-axis-text", x: pl + iw, y: H - 6, "text-anchor": "end" }, svg);
    last.textContent = fmt(bins[bins.length - 1].upper);
  }

  function categorical(host, stats) {
    var vals = (stats && stats.top_values) || [];
    if (!vals.length) { host.innerHTML = '<div class="mp-empty">No values.</div>'; return; }
    var rowH = 22, gap = 6, W = 420, labelW = 120, barW = W - labelW - 46;
    var H = vals.length * (rowH + gap);
    var maxC = vals.reduce(function (a, b) { return Math.max(a, b.count); }, 0) || 1;
    var svg = el("svg", { viewBox: "0 0 " + W + " " + H }, host);
    vals.forEach(function (v, i) {
      var y = i * (rowH + gap);
      var label = String(v.value);
      var t = el("text", { class: "mp-axis-text", x: 0, y: y + rowH * 0.7 }, svg);
      t.textContent = label.length > 18 ? label.slice(0, 17) + "…" : label;
      var w = Math.max((v.count / maxC) * barW, 2);
      var rect = el("rect", { class: "mp-bar", x: labelW, y: y, width: w, height: rowH, rx: 2 }, svg);
      var cnt = el("text", { class: "mp-axis-text", x: labelW + w + 6, y: y + rowH * 0.7 }, svg);
      cnt.textContent = v.count + " (" + fmt(v.percentage) + "%)";
      hover(rect, "<b>" + label + "</b><br>count: " + v.count + " &middot; " + fmt(v.percentage) + "%");
    });
  }

  function missing(host) {
    var cols = (DATA.missing && DATA.missing.columns) || [];
    if (!cols.length) { host.innerHTML = '<div class="mp-empty">No columns.</div>'; return; }
    var rowH = 20, gap = 6, W = 640, labelW = 150, barW = W - labelW - 70;
    var H = cols.length * (rowH + gap);
    var svg = el("svg", { viewBox: "0 0 " + W + " " + H }, host);
    cols.forEach(function (c, i) {
      var y = i * (rowH + gap);
      var t = el("text", { class: "mp-axis-text", x: 0, y: y + rowH * 0.7 }, svg);
      var nm = c.name.length > 22 ? c.name.slice(0, 21) + "…" : c.name;
      t.textContent = nm;
      el("rect", { x: labelW, y: y, width: barW, height: rowH, rx: 2, fill: "#eef0f3" }, svg);
      var w = Math.max((c.missing_pct / 100) * barW, c.missing_count > 0 ? 2 : 0);
      var color = c.missing_pct > 50 ? "#d6453d" : c.missing_pct > 20 ? "#d98324" : "#3b6fd4";
      if (w > 0) {
        var rect = el("rect", { x: labelW, y: y, width: w, height: rowH, rx: 2, fill: color }, svg);
        hover(rect, "<b>" + c.name + "</b><br>missing: " + c.missing_count + " (" + fmt(c.missing_pct) + "%)");
      }
      var lbl = el("text", { class: "mp-axis-text", x: labelW + barW + 6, y: y + rowH * 0.7 }, svg);
      lbl.textContent = fmt(c.missing_pct) + "%";
    });
  }

  function corrColor(v) {
    // Diverging blue (−1) → white (0) → red (+1).
    if (v === null || v === undefined) return "#eef0f3";
    var t = Math.max(-1, Math.min(1, v));
    var r, g, b;
    if (t >= 0) { r = 255; g = Math.round(255 - t * 178); b = Math.round(255 - t * 184); }
    else { r = Math.round(255 + t * 208); g = Math.round(255 + t * 144); b = 255; }
    return "rgb(" + r + "," + g + "," + b + ")";
  }

  function correlation(host) {
    var corr = DATA.correlations || {};
    var cols = corr.columns || [];
    var mat = corr.matrix || [];
    if (cols.length < 2) { host.innerHTML = '<div class="mp-empty">Need at least two numeric columns.</div>'; return; }
    var n = cols.length;
    var labelW = 90, top = 14, cell = Math.max(Math.min(34, Math.floor(420 / n)), 16);
    var W = labelW + n * cell + 8, H = top + labelW + n * cell + 8;
    var svg = el("svg", { viewBox: "0 0 " + W + " " + H }, host);
    cols.forEach(function (name, j) {
      var x = labelW + j * cell + cell / 2;
      var t = el("text", { class: "mp-axis-text", x: x, y: top + labelW - 4, transform: "rotate(-55 " + x + " " + (top + labelW - 4) + ")", "text-anchor": "start" }, svg);
      t.textContent = name.length > 12 ? name.slice(0, 11) + "…" : name;
    });
    for (var i = 0; i < n; i++) {
      var ly = top + labelW + i * cell + cell * 0.65;
      var lt = el("text", { class: "mp-axis-text", x: labelW - 6, y: ly, "text-anchor": "end" }, svg);
      lt.textContent = cols[i].length > 12 ? cols[i].slice(0, 11) + "…" : cols[i];
      for (var j2 = 0; j2 < n; j2++) {
        var v = mat[i] ? mat[i][j2] : null;
        var rx = labelW + j2 * cell, ry = top + labelW + i * cell;
        var rect = el("rect", { class: "mp-cell", x: rx, y: ry, width: cell, height: cell, fill: corrColor(v) }, svg);
        hover(rect, "<b>" + cols[i] + " × " + cols[j2] + "</b><br>r = " + (v === null ? "n/a" : fmt(v)));
        if (cell >= 28 && v !== null && v !== undefined) {
          var txt = el("text", { x: rx + cell / 2, y: ry + cell * 0.62, "text-anchor": "middle", "font-size": "9", fill: Math.abs(v) > 0.6 ? "#fff" : "#444" }, svg);
          txt.textContent = (Math.round(v * 100) / 100).toFixed(2);
        }
      }
    }
  }

  document.querySelectorAll("[data-chart]").forEach(function (host) {
    var kind = host.getAttribute("data-chart");
    var col = host.getAttribute("data-col");
    try {
      if (kind === "hist") histogram(host, NUM[col]);
      else if (kind === "cat") categorical(host, CAT[col]);
      else if (kind === "missing") missing(host);
      else if (kind === "corr") correlation(host);
    } catch (err) {
      host.innerHTML = '<div class="mp-empty">Chart error.</div>';
    }
  });
})();
