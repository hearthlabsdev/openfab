//#region \0rolldown/runtime.js
var e = (e, t, n) => () => {
	if (n) throw n[0];
	try {
		return e && (t = e(e = 0)), t;
	} catch (e) {
		throw n = [e], e;
	}
}, t = (e, t) => () => (t || (e((t = { exports: {} }).exports, t), e = null), t.exports), n, r, i, a, o, s, c, l, u = e((() => {
	n = globalThis, r = n.ShadowRoot && (n.ShadyCSS === void 0 || n.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype, i = Symbol(), a = /* @__PURE__ */ new WeakMap(), o = class {
		constructor(e, t, n) {
			if (this._$cssResult$ = !0, n !== i) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
			this.cssText = e, this.t = t;
		}
		get styleSheet() {
			let e = this.o, t = this.t;
			if (r && e === void 0) {
				let n = t !== void 0 && t.length === 1;
				n && (e = a.get(t)), e === void 0 && ((this.o = e = new CSSStyleSheet()).replaceSync(this.cssText), n && a.set(t, e));
			}
			return e;
		}
		toString() {
			return this.cssText;
		}
	}, s = (e) => new o(typeof e == "string" ? e : e + "", void 0, i), c = (e, t) => {
		if (r) e.adoptedStyleSheets = t.map((e) => e instanceof CSSStyleSheet ? e : e.styleSheet);
		else for (let r of t) {
			let t = document.createElement("style"), i = n.litNonce;
			i !== void 0 && t.setAttribute("nonce", i), t.textContent = r.cssText, e.appendChild(t);
		}
	}, l = r ? (e) => e : (e) => e instanceof CSSStyleSheet ? ((e) => {
		let t = "";
		for (let n of e.cssRules) t += n.cssText;
		return s(t);
	})(e) : e;
})), d, f, p, m, ee, te, h, ne, re, ie, g, _, v, ae, y, b = e((() => {
	u(), {is: d, defineProperty: f, getOwnPropertyDescriptor: p, getOwnPropertyNames: m, getOwnPropertySymbols: ee, getPrototypeOf: te} = Object, h = globalThis, ne = h.trustedTypes, re = ne ? ne.emptyScript : "", ie = h.reactiveElementPolyfillSupport, g = (e, t) => e, _ = {
		toAttribute(e, t) {
			switch (t) {
				case Boolean:
					e = e ? re : null;
					break;
				case Object:
				case Array: e = e == null ? e : JSON.stringify(e);
			}
			return e;
		},
		fromAttribute(e, t) {
			let n = e;
			switch (t) {
				case Boolean:
					n = e !== null;
					break;
				case Number:
					n = e === null ? null : Number(e);
					break;
				case Object:
				case Array: try {
					n = JSON.parse(e);
				} catch {
					n = null;
				}
			}
			return n;
		}
	}, v = (e, t) => !d(e, t), ae = {
		attribute: !0,
		type: String,
		converter: _,
		reflect: !1,
		useDefault: !1,
		hasChanged: v
	}, Symbol.metadata ??= Symbol("metadata"), h.litPropertyMetadata ??= /* @__PURE__ */ new WeakMap(), y = class extends HTMLElement {
		static addInitializer(e) {
			this._$Ei(), (this.l ??= []).push(e);
		}
		static get observedAttributes() {
			return this.finalize(), this._$Eh && [...this._$Eh.keys()];
		}
		static createProperty(e, t = ae) {
			if (t.state && (t.attribute = !1), this._$Ei(), this.prototype.hasOwnProperty(e) && ((t = Object.create(t)).wrapped = !0), this.elementProperties.set(e, t), !t.noAccessor) {
				let n = Symbol(), r = this.getPropertyDescriptor(e, n, t);
				r !== void 0 && f(this.prototype, e, r);
			}
		}
		static getPropertyDescriptor(e, t, n) {
			let { get: r, set: i } = p(this.prototype, e) ?? {
				get() {
					return this[t];
				},
				set(e) {
					this[t] = e;
				}
			};
			return {
				get: r,
				set(t) {
					let a = r?.call(this);
					i?.call(this, t), this.requestUpdate(e, a, n);
				},
				configurable: !0,
				enumerable: !0
			};
		}
		static getPropertyOptions(e) {
			return this.elementProperties.get(e) ?? ae;
		}
		static _$Ei() {
			if (this.hasOwnProperty(g("elementProperties"))) return;
			let e = te(this);
			e.finalize(), e.l !== void 0 && (this.l = [...e.l]), this.elementProperties = new Map(e.elementProperties);
		}
		static finalize() {
			if (this.hasOwnProperty(g("finalized"))) return;
			if (this.finalized = !0, this._$Ei(), this.hasOwnProperty(g("properties"))) {
				let e = this.properties, t = [...m(e), ...ee(e)];
				for (let n of t) this.createProperty(n, e[n]);
			}
			let e = this[Symbol.metadata];
			if (e !== null) {
				let t = litPropertyMetadata.get(e);
				if (t !== void 0) for (let [e, n] of t) this.elementProperties.set(e, n);
			}
			this._$Eh = /* @__PURE__ */ new Map();
			for (let [e, t] of this.elementProperties) {
				let n = this._$Eu(e, t);
				n !== void 0 && this._$Eh.set(n, e);
			}
			this.elementStyles = this.finalizeStyles(this.styles);
		}
		static finalizeStyles(e) {
			let t = [];
			if (Array.isArray(e)) {
				let n = new Set(e.flat(1 / 0).reverse());
				for (let e of n) t.unshift(l(e));
			} else e !== void 0 && t.push(l(e));
			return t;
		}
		static _$Eu(e, t) {
			let n = t.attribute;
			return !1 === n ? void 0 : typeof n == "string" ? n : typeof e == "string" ? e.toLowerCase() : void 0;
		}
		constructor() {
			super(), this._$Ep = void 0, this.isUpdatePending = !1, this.hasUpdated = !1, this._$Em = null, this._$Ev();
		}
		_$Ev() {
			this._$ES = new Promise((e) => this.enableUpdating = e), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((e) => e(this));
		}
		addController(e) {
			(this._$EO ??= /* @__PURE__ */ new Set()).add(e), this.renderRoot !== void 0 && this.isConnected && e.hostConnected?.();
		}
		removeController(e) {
			this._$EO?.delete(e);
		}
		_$E_() {
			let e = /* @__PURE__ */ new Map(), t = this.constructor.elementProperties;
			for (let n of t.keys()) this.hasOwnProperty(n) && (e.set(n, this[n]), delete this[n]);
			e.size > 0 && (this._$Ep = e);
		}
		createRenderRoot() {
			let e = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
			return c(e, this.constructor.elementStyles), e;
		}
		connectedCallback() {
			this.renderRoot ??= this.createRenderRoot(), this.enableUpdating(!0), this._$EO?.forEach((e) => e.hostConnected?.());
		}
		enableUpdating(e) {}
		disconnectedCallback() {
			this._$EO?.forEach((e) => e.hostDisconnected?.());
		}
		attributeChangedCallback(e, t, n) {
			this._$AK(e, n);
		}
		_$ET(e, t) {
			let n = this.constructor.elementProperties.get(e), r = this.constructor._$Eu(e, n);
			if (r !== void 0 && !0 === n.reflect) {
				let i = (n.converter?.toAttribute === void 0 ? _ : n.converter).toAttribute(t, n.type);
				this._$Em = e, i == null ? this.removeAttribute(r) : this.setAttribute(r, i), this._$Em = null;
			}
		}
		_$AK(e, t) {
			let n = this.constructor, r = n._$Eh.get(e);
			if (r !== void 0 && this._$Em !== r) {
				let e = n.getPropertyOptions(r), i = typeof e.converter == "function" ? { fromAttribute: e.converter } : e.converter?.fromAttribute === void 0 ? _ : e.converter;
				this._$Em = r;
				let a = i.fromAttribute(t, e.type);
				this[r] = a ?? this._$Ej?.get(r) ?? a, this._$Em = null;
			}
		}
		requestUpdate(e, t, n, r = !1, i) {
			if (e !== void 0) {
				let a = this.constructor;
				if (!1 === r && (i = this[e]), n ??= a.getPropertyOptions(e), !((n.hasChanged ?? v)(i, t) || n.useDefault && n.reflect && i === this._$Ej?.get(e) && !this.hasAttribute(a._$Eu(e, n)))) return;
				this.C(e, t, n);
			}
			!1 === this.isUpdatePending && (this._$ES = this._$EP());
		}
		C(e, t, { useDefault: n, reflect: r, wrapped: i }, a) {
			n && !(this._$Ej ??= /* @__PURE__ */ new Map()).has(e) && (this._$Ej.set(e, a ?? t ?? this[e]), !0 !== i || a !== void 0) || (this._$AL.has(e) || (this.hasUpdated || n || (t = void 0), this._$AL.set(e, t)), !0 === r && this._$Em !== e && (this._$Eq ??= /* @__PURE__ */ new Set()).add(e));
		}
		async _$EP() {
			this.isUpdatePending = !0;
			try {
				await this._$ES;
			} catch (e) {
				Promise.reject(e);
			}
			let e = this.scheduleUpdate();
			return e != null && await e, !this.isUpdatePending;
		}
		scheduleUpdate() {
			return this.performUpdate();
		}
		performUpdate() {
			if (!this.isUpdatePending) return;
			if (!this.hasUpdated) {
				if (this.renderRoot ??= this.createRenderRoot(), this._$Ep) {
					for (let [e, t] of this._$Ep) this[e] = t;
					this._$Ep = void 0;
				}
				let e = this.constructor.elementProperties;
				if (e.size > 0) for (let [t, n] of e) {
					let { wrapped: e } = n, r = this[t];
					!0 !== e || this._$AL.has(t) || r === void 0 || this.C(t, void 0, n, r);
				}
			}
			let e = !1, t = this._$AL;
			try {
				e = this.shouldUpdate(t), e ? (this.willUpdate(t), this._$EO?.forEach((e) => e.hostUpdate?.()), this.update(t)) : this._$EM();
			} catch (t) {
				throw e = !1, this._$EM(), t;
			}
			e && this._$AE(t);
		}
		willUpdate(e) {}
		_$AE(e) {
			this._$EO?.forEach((e) => e.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = !0, this.firstUpdated(e)), this.updated(e);
		}
		_$EM() {
			this._$AL = /* @__PURE__ */ new Map(), this.isUpdatePending = !1;
		}
		get updateComplete() {
			return this.getUpdateComplete();
		}
		getUpdateComplete() {
			return this._$ES;
		}
		shouldUpdate(e) {
			return !0;
		}
		update(e) {
			this._$Eq &&= this._$Eq.forEach((e) => this._$ET(e, this[e])), this._$EM();
		}
		updated(e) {}
		firstUpdated(e) {}
	}, y.elementStyles = [], y.shadowRootOptions = { mode: "open" }, y[g("elementProperties")] = /* @__PURE__ */ new Map(), y[g("finalized")] = /* @__PURE__ */ new Map(), ie?.({ ReactiveElement: y }), (h.reactiveElementVersions ??= []).push("2.1.2");
}));
//#endregion
//#region node_modules/lit-html/lit-html.js
function oe(e, t) {
	if (!j(e) || !e.hasOwnProperty("raw")) throw Error("invalid template strings array");
	return se === void 0 ? t : se.createHTML(t);
}
function x(e, t, n = e, r) {
	if (t === R) return t;
	let i = r === void 0 ? n._$Cl : n._$Co?.[r], a = A(t) ? void 0 : t._$litDirective$;
	return i?.constructor !== a && (i?._$AO?.(!1), a === void 0 ? i = void 0 : (i = new a(e), i._$AT(e, n, r)), r === void 0 ? n._$Cl = i : (n._$Co ??= [])[r] = i), i !== void 0 && (t = x(e, i._$AS(e, t.values), i, r)), t;
}
var S, C, w, se, T, E, D, ce, O, k, A, j, M, N, P, le, ue, F, de, fe, pe, I, L, R, z, me, B, he, V, ge, H, U, _e, ve, ye, be, xe, Se, Ce, W = e((() => {
	S = globalThis, C = (e) => e, w = S.trustedTypes, se = w ? w.createPolicy("lit-html", { createHTML: (e) => e }) : void 0, T = "$lit$", E = `lit$${Math.random().toFixed(9).slice(2)}$`, D = "?" + E, ce = `<${D}>`, O = document, k = () => O.createComment(""), A = (e) => e === null || typeof e != "object" && typeof e != "function", j = Array.isArray, M = (e) => j(e) || typeof e?.[Symbol.iterator] == "function", N = "[ 	\n\f\r]", P = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g, le = /-->/g, ue = />/g, F = RegExp(`>|${N}(?:([^\\s"'>=/]+)(${N}*=${N}*(?:[^ \t\n\f\r"'\`<>=]|("|')|))|$)`, "g"), de = /'/g, fe = /"/g, pe = /^(?:script|style|textarea|title)$/i, I = (e) => (t, ...n) => ({
		_$litType$: e,
		strings: t,
		values: n
	}), L = I(1), I(2), I(3), R = Symbol.for("lit-noChange"), z = Symbol.for("lit-nothing"), me = /* @__PURE__ */ new WeakMap(), B = O.createTreeWalker(O, 129), he = (e, t) => {
		let n = e.length - 1, r = [], i, a = t === 2 ? "<svg>" : t === 3 ? "<math>" : "", o = P;
		for (let t = 0; t < n; t++) {
			let n = e[t], s, c, l = -1, u = 0;
			for (; u < n.length && (o.lastIndex = u, c = o.exec(n), c !== null);) u = o.lastIndex, o === P ? c[1] === "!--" ? o = le : c[1] === void 0 ? c[2] === void 0 ? c[3] !== void 0 && (o = F) : (pe.test(c[2]) && (i = RegExp("</" + c[2], "g")), o = F) : o = ue : o === F ? c[0] === ">" ? (o = i ?? P, l = -1) : c[1] === void 0 ? l = -2 : (l = o.lastIndex - c[2].length, s = c[1], o = c[3] === void 0 ? F : c[3] === "\"" ? fe : de) : o === fe || o === de ? o = F : o === le || o === ue ? o = P : (o = F, i = void 0);
			let d = o === F && e[t + 1].startsWith("/>") ? " " : "";
			a += o === P ? n + ce : l >= 0 ? (r.push(s), n.slice(0, l) + T + n.slice(l) + E + d) : n + E + (l === -2 ? t : d);
		}
		return [oe(e, a + (e[n] || "<?>") + (t === 2 ? "</svg>" : t === 3 ? "</math>" : "")), r];
	}, V = class e {
		constructor({ strings: t, _$litType$: n }, r) {
			let i;
			this.parts = [];
			let a = 0, o = 0, s = t.length - 1, c = this.parts, [l, u] = he(t, n);
			if (this.el = e.createElement(l, r), B.currentNode = this.el.content, n === 2 || n === 3) {
				let e = this.el.content.firstChild;
				e.replaceWith(...e.childNodes);
			}
			for (; (i = B.nextNode()) !== null && c.length < s;) {
				if (i.nodeType === 1) {
					if (i.hasAttributes()) for (let e of i.getAttributeNames()) if (e.endsWith(T)) {
						let t = u[o++], n = i.getAttribute(e).split(E), r = /([.?@])?(.*)/.exec(t);
						c.push({
							type: 1,
							index: a,
							name: r[2],
							strings: n,
							ctor: r[1] === "." ? _e : r[1] === "?" ? ve : r[1] === "@" ? ye : U
						}), i.removeAttribute(e);
					} else e.startsWith(E) && (c.push({
						type: 6,
						index: a
					}), i.removeAttribute(e));
					if (pe.test(i.tagName)) {
						let e = i.textContent.split(E), t = e.length - 1;
						if (t > 0) {
							i.textContent = w ? w.emptyScript : "";
							for (let n = 0; n < t; n++) i.append(e[n], k()), B.nextNode(), c.push({
								type: 2,
								index: ++a
							});
							i.append(e[t], k());
						}
					}
				} else if (i.nodeType === 8) {
					if (i.data === D) c.push({
						type: 2,
						index: a
					});
					else {
						let e = -1;
						for (; (e = i.data.indexOf(E, e + 1)) !== -1;) c.push({
							type: 7,
							index: a
						}), e += E.length - 1;
					}
				}
				a++;
			}
		}
		static createElement(e, t) {
			let n = O.createElement("template");
			return n.innerHTML = e, n;
		}
	}, ge = class {
		constructor(e, t) {
			this._$AV = [], this._$AN = void 0, this._$AD = e, this._$AM = t;
		}
		get parentNode() {
			return this._$AM.parentNode;
		}
		get _$AU() {
			return this._$AM._$AU;
		}
		u(e) {
			let { el: { content: t }, parts: n } = this._$AD, r = (e?.creationScope ?? O).importNode(t, !0);
			B.currentNode = r;
			let i = B.nextNode(), a = 0, o = 0, s = n[0];
			for (; s !== void 0;) {
				if (a === s.index) {
					let t;
					s.type === 2 ? t = new H(i, i.nextSibling, this, e) : s.type === 1 ? t = new s.ctor(i, s.name, s.strings, this, e) : s.type === 6 && (t = new be(i, this, e)), this._$AV.push(t), s = n[++o];
				}
				a !== s?.index && (i = B.nextNode(), a++);
			}
			return B.currentNode = O, r;
		}
		p(e) {
			let t = 0;
			for (let n of this._$AV) n !== void 0 && (n.strings === void 0 ? n._$AI(e[t]) : (n._$AI(e, n, t), t += n.strings.length - 2)), t++;
		}
	}, H = class e {
		get _$AU() {
			return this._$AM?._$AU ?? this._$Cv;
		}
		constructor(e, t, n, r) {
			this.type = 2, this._$AH = z, this._$AN = void 0, this._$AA = e, this._$AB = t, this._$AM = n, this.options = r, this._$Cv = r?.isConnected ?? !0;
		}
		get parentNode() {
			let e = this._$AA.parentNode, t = this._$AM;
			return t !== void 0 && e?.nodeType === 11 && (e = t.parentNode), e;
		}
		get startNode() {
			return this._$AA;
		}
		get endNode() {
			return this._$AB;
		}
		_$AI(e, t = this) {
			e = x(this, e, t), A(e) ? e === z || e == null || e === "" ? (this._$AH !== z && this._$AR(), this._$AH = z) : e !== this._$AH && e !== R && this._(e) : e._$litType$ === void 0 ? e.nodeType === void 0 ? M(e) ? this.k(e) : this._(e) : this.T(e) : this.$(e);
		}
		O(e) {
			return this._$AA.parentNode.insertBefore(e, this._$AB);
		}
		T(e) {
			this._$AH !== e && (this._$AR(), this._$AH = this.O(e));
		}
		_(e) {
			this._$AH !== z && A(this._$AH) ? this._$AA.nextSibling.data = e : this.T(O.createTextNode(e)), this._$AH = e;
		}
		$(e) {
			let { values: t, _$litType$: n } = e, r = typeof n == "number" ? this._$AC(e) : (n.el === void 0 && (n.el = V.createElement(oe(n.h, n.h[0]), this.options)), n);
			if (this._$AH?._$AD === r) this._$AH.p(t);
			else {
				let e = new ge(r, this), n = e.u(this.options);
				e.p(t), this.T(n), this._$AH = e;
			}
		}
		_$AC(e) {
			let t = me.get(e.strings);
			return t === void 0 && me.set(e.strings, t = new V(e)), t;
		}
		k(t) {
			j(this._$AH) || (this._$AH = [], this._$AR());
			let n = this._$AH, r, i = 0;
			for (let a of t) i === n.length ? n.push(r = new e(this.O(k()), this.O(k()), this, this.options)) : r = n[i], r._$AI(a), i++;
			i < n.length && (this._$AR(r && r._$AB.nextSibling, i), n.length = i);
		}
		_$AR(e = this._$AA.nextSibling, t) {
			for (this._$AP?.(!1, !0, t); e !== this._$AB;) {
				let t = C(e).nextSibling;
				C(e).remove(), e = t;
			}
		}
		setConnected(e) {
			this._$AM === void 0 && (this._$Cv = e, this._$AP?.(e));
		}
	}, U = class {
		get tagName() {
			return this.element.tagName;
		}
		get _$AU() {
			return this._$AM._$AU;
		}
		constructor(e, t, n, r, i) {
			this.type = 1, this._$AH = z, this._$AN = void 0, this.element = e, this.name = t, this._$AM = r, this.options = i, n.length > 2 || n[0] !== "" || n[1] !== "" ? (this._$AH = Array(n.length - 1).fill(/* @__PURE__ */ new String()), this.strings = n) : this._$AH = z;
		}
		_$AI(e, t = this, n, r) {
			let i = this.strings, a = !1;
			if (i === void 0) e = x(this, e, t, 0), a = !A(e) || e !== this._$AH && e !== R, a && (this._$AH = e);
			else {
				let r = e, o, s;
				for (e = i[0], o = 0; o < i.length - 1; o++) s = x(this, r[n + o], t, o), s === R && (s = this._$AH[o]), a ||= !A(s) || s !== this._$AH[o], s === z ? e = z : e !== z && (e += (s ?? "") + i[o + 1]), this._$AH[o] = s;
			}
			a && !r && this.j(e);
		}
		j(e) {
			e === z ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, e ?? "");
		}
	}, _e = class extends U {
		constructor() {
			super(...arguments), this.type = 3;
		}
		j(e) {
			this.element[this.name] = e === z ? void 0 : e;
		}
	}, ve = class extends U {
		constructor() {
			super(...arguments), this.type = 4;
		}
		j(e) {
			this.element.toggleAttribute(this.name, !!e && e !== z);
		}
	}, ye = class extends U {
		constructor(e, t, n, r, i) {
			super(e, t, n, r, i), this.type = 5;
		}
		_$AI(e, t = this) {
			if ((e = x(this, e, t, 0) ?? z) === R) return;
			let n = this._$AH, r = e === z && n !== z || e.capture !== n.capture || e.once !== n.once || e.passive !== n.passive, i = e !== z && (n === z || r);
			r && this.element.removeEventListener(this.name, this, n), i && this.element.addEventListener(this.name, this, e), this._$AH = e;
		}
		handleEvent(e) {
			typeof this._$AH == "function" ? this._$AH.call(this.options?.host ?? this.element, e) : this._$AH.handleEvent(e);
		}
	}, be = class {
		constructor(e, t, n) {
			this.element = e, this.type = 6, this._$AN = void 0, this._$AM = t, this.options = n;
		}
		get _$AU() {
			return this._$AM._$AU;
		}
		_$AI(e) {
			x(this, e);
		}
	}, xe = {
		M: T,
		P: E,
		A: D,
		C: 1,
		L: he,
		R: ge,
		D: M,
		V: x,
		I: H,
		H: U,
		N: ve,
		U: ye,
		B: _e,
		F: be
	}, Se = S.litHtmlPolyfillSupport, Se?.(V, H), (S.litHtmlVersions ??= []).push("3.3.3"), Ce = (e, t, n) => {
		let r = n?.renderBefore ?? t, i = r._$litPart$;
		if (i === void 0) {
			let e = n?.renderBefore ?? null;
			r._$litPart$ = i = new H(t.insertBefore(k(), e), e, void 0, n ?? {});
		}
		return i._$AI(e), i;
	};
})), G, K, we, Te = e((() => {
	b(), b(), W(), W(), G = globalThis, K = class extends y {
		constructor() {
			super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
		}
		createRenderRoot() {
			let e = super.createRenderRoot();
			return this.renderOptions.renderBefore ??= e.firstChild, e;
		}
		update(e) {
			let t = this.render();
			this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(e), this._$Do = Ce(t, this.renderRoot, this.renderOptions);
		}
		connectedCallback() {
			super.connectedCallback(), this._$Do?.setConnected(!0);
		}
		disconnectedCallback() {
			super.disconnectedCallback(), this._$Do?.setConnected(!1);
		}
		render() {
			return R;
		}
	}, K._$litElement$ = !0, K.finalized = !0, G.litElementHydrateSupport?.({ LitElement: K }), we = G.litElementPolyfillSupport, we?.({ LitElement: K }), (G.litElementVersions ??= []).push("4.2.2");
})), Ee = e((() => {})), De = e((() => {
	b(), W(), Te(), Ee();
})), Oe, ke = e((() => {
	Oe = (e) => (t, n) => {
		n === void 0 ? customElements.define(e, t) : n.addInitializer(() => {
			customElements.define(e, t);
		});
	};
}));
//#endregion
//#region node_modules/@lit/reactive-element/decorators/property.js
function q(e) {
	return (t, n) => typeof n == "object" ? je(e, t, n) : ((e, t, n) => {
		let r = t.hasOwnProperty(n);
		return t.constructor.createProperty(n, e), r ? Object.getOwnPropertyDescriptor(t, n) : void 0;
	})(e, t, n);
}
var Ae, je, Me = e((() => {
	b(), Ae = {
		attribute: !0,
		type: String,
		converter: _,
		reflect: !1,
		hasChanged: v
	}, je = (e = Ae, t, n) => {
		let { kind: r, metadata: i } = n, a = globalThis.litPropertyMetadata.get(i);
		if (a === void 0 && globalThis.litPropertyMetadata.set(i, a = /* @__PURE__ */ new Map()), r === "setter" && ((e = Object.create(e)).wrapped = !0), a.set(n.name, e), r === "accessor") {
			let { name: r } = n;
			return {
				set(n) {
					let i = t.get.call(this);
					t.set.call(this, n), this.requestUpdate(r, i, e, !0, n);
				},
				init(t) {
					return t !== void 0 && this.C(r, void 0, e, t), t;
				}
			};
		}
		if (r === "setter") {
			let { name: r } = n;
			return function(n) {
				let i = this[r];
				t.call(this, n), this.requestUpdate(r, i, e, !0, n);
			};
		}
		throw Error("Unsupported decorator location: " + r);
	};
}));
//#endregion
//#region node_modules/@lit/reactive-element/decorators/state.js
function J(e) {
	return q({
		...e,
		state: !0,
		attribute: !1
	});
}
var Ne = e((() => {
	Me();
})), Pe = e((() => {})), Fe = e((() => {})), Ie = e((() => {})), Le = e((() => {})), Re = e((() => {})), ze = e((() => {})), Be = e((() => {
	ke(), Me(), Ne(), Pe(), Fe(), Ie(), Le(), Re(), ze();
})), Ve, He, Ue, We = e((() => {
	Ve = {
		ATTRIBUTE: 1,
		CHILD: 2,
		PROPERTY: 3,
		BOOLEAN_ATTRIBUTE: 4,
		EVENT: 5,
		ELEMENT: 6
	}, He = (e) => (...t) => ({
		_$litDirective$: e,
		values: t
	}), Ue = class {
		constructor(e) {}
		get _$AU() {
			return this._$AM._$AU;
		}
		_$AT(e, t, n) {
			this._$Ct = e, this._$AM = t, this._$Ci = n;
		}
		_$AS(e, t) {
			return this.update(e, t);
		}
		update(e, t) {
			return this.render(...t);
		}
	};
})), Ge, Ke, qe, Y, X, Je, Ye, Xe, Z, Ze = e((() => {
	W(), {I: Ge} = xe, Ke = (e) => e, qe = () => document.createComment(""), Y = (e, t, n) => {
		let r = e._$AA.parentNode, i = t === void 0 ? e._$AB : t._$AA;
		if (n === void 0) {
			let t = r.insertBefore(qe(), i), a = r.insertBefore(qe(), i);
			n = new Ge(t, a, e, e.options);
		} else {
			let t = n._$AB.nextSibling, a = n._$AM, o = a !== e;
			if (o) {
				let t;
				n._$AQ?.(e), n._$AM = e, n._$AP !== void 0 && (t = e._$AU) !== a._$AU && n._$AP(t);
			}
			if (t !== i || o) {
				let e = n._$AA;
				for (; e !== t;) {
					let t = Ke(e).nextSibling;
					Ke(r).insertBefore(e, i), e = t;
				}
			}
		}
		return n;
	}, X = (e, t, n = e) => (e._$AI(t, n), e), Je = {}, Ye = (e, t = Je) => e._$AH = t, Xe = (e) => e._$AH, Z = (e) => {
		e._$AR(), e._$AA.remove();
	};
})), Qe, $e, et = e((() => {
	W(), We(), Ze(), Qe = (e, t, n) => {
		let r = /* @__PURE__ */ new Map();
		for (let i = t; i <= n; i++) r.set(e[i], i);
		return r;
	}, $e = He(class extends Ue {
		constructor(e) {
			if (super(e), e.type !== Ve.CHILD) throw Error("repeat() can only be used in text expressions");
		}
		dt(e, t, n) {
			let r;
			n === void 0 ? n = t : t !== void 0 && (r = t);
			let i = [], a = [], o = 0;
			for (let t of e) i[o] = r ? r(t, o) : o, a[o] = n(t, o), o++;
			return {
				values: a,
				keys: i
			};
		}
		render(e, t, n) {
			return this.dt(e, t, n).values;
		}
		update(e, [t, n, r]) {
			let i = Xe(e), { values: a, keys: o } = this.dt(t, n, r);
			if (!Array.isArray(i)) return this.ut = o, a;
			let s = this.ut ??= [], c = [], l, u, d = 0, f = i.length - 1, p = 0, m = a.length - 1;
			for (; d <= f && p <= m;) if (i[d] === null) d++;
			else if (i[f] === null) f--;
			else if (s[d] === o[p]) c[p] = X(i[d], a[p]), d++, p++;
			else if (s[f] === o[m]) c[m] = X(i[f], a[m]), f--, m--;
			else if (s[d] === o[m]) c[m] = X(i[d], a[m]), Y(e, c[m + 1], i[d]), d++, m--;
			else if (s[f] === o[p]) c[p] = X(i[f], a[p]), Y(e, i[d], i[f]), f--, p++;
			else if (l === void 0 && (l = Qe(o, p, m), u = Qe(s, d, f)), l.has(s[d])) {
				if (l.has(s[f])) {
					let t = u.get(o[p]), n = t === void 0 ? null : i[t];
					if (n === null) {
						let t = Y(e, i[d]);
						X(t, a[p]), c[p] = t;
					} else c[p] = X(n, a[p]), Y(e, i[d], n), i[t] = null;
					p++;
				} else Z(i[f]), f--;
			} else Z(i[d]), d++;
			for (; p <= m;) {
				let t = Y(e, c[m + 1]);
				X(t, a[p]), c[p++] = t;
			}
			for (; d <= f;) {
				let e = i[d++];
				e !== null && Z(e);
			}
			return this.ut = o, Ye(e, c), R;
		}
	});
})), tt = e((() => {
	et();
}));
//#endregion
//#region \0@oxc-project+runtime@0.153.0/helpers/esm/decorate.js
function Q(e, t, n, r) {
	var i = arguments.length, a = i < 3 ? t : r === null ? r = Object.getOwnPropertyDescriptor(t, n) : r, o;
	if (typeof Reflect == "object" && typeof Reflect.decorate == "function") a = Reflect.decorate(e, t, n, r);
	else for (var s = e.length - 1; s >= 0; s--) (o = e[s]) && (a = (i < 3 ? o(a) : i > 3 ? o(t, n, a) : o(t, n)) || a);
	return i > 3 && a && Object.defineProperty(t, n, a), a;
}
var nt = e((() => {})), $, rt = e((() => {
	De(), Be(), tt(), nt(), $ = class extends K {
		constructor(...e) {
			super(...e), this.printerId = "", this.streamUrl = "", this.connectionState = "Connecting", this.errorMessage = "", this.shouldReconnect = !0;
		}
		connectedCallback() {
			super.connectedCallback(), this.shouldReconnect = !0, this.connectWebSocket();
		}
		disconnectedCallback() {
			this.shouldReconnect = !1, this.ws?.close(), super.disconnectedCallback();
		}
		connectWebSocket() {
			if (!this.streamUrl && !this.printerId) return;
			let e = window.location.protocol === "https:" ? "wss:" : "ws:";
			this.streamUrl ||= `/devices/${this.printerId}/status`;
			let t = this.streamUrl.startsWith("ws") ? this.streamUrl : `${e}//${window.location.host}${this.streamUrl}`;
			this.ws = new WebSocket(t);
			let n = 3e3, r = setTimeout(() => {
				this.ws.readyState !== WebSocket.OPEN && (console.warn(`WebSocket connection timed out after ${n}ms`), this.shouldReconnect = !1, this.errorMessage = "Connection timed out: Printer stream is unreachable.", this.connectionState = "Disconnected", this.ws.close());
			}, n);
			this.ws.onopen = () => {
				clearTimeout(r), this.connectionState = "Connected", this.errorMessage = "";
			}, this.ws.onmessage = (e) => {
				let t = JSON.parse(e.data);
				"Err" in t ? (console.error("Backend error received:", t.Err), this.shouldReconnect = !1, this.errorMessage = t.Err, this.ws?.close(1e3, "Fatal error from backend")) : "StatusUpdate" in t ? this.printerStatus = t.StatusUpdate : "PrintStarted" in t || "JobUpdate" in t;
			}, this.ws.onerror = (e) => {
				console.error("WebSocket connection failed (Device likely not found):", e), this.connectionState = "Disconnected", this.errorMessage = "Unable to connect to printer stream (Device not found or offline).";
			}, this.ws.onclose = () => {
				this.connectionState = "Disconnected", this.shouldReconnect && setTimeout(() => this.connectWebSocket(), 3e3);
			};
		}
		formatTime(e) {
			if (e <= 0) return "Unknown";
			let t = Math.floor(e / 3600), n = Math.floor(e % 3600 / 60), r = e % 60;
			return t > 0 ? `${t}h ${n}m` : `${n}m ${r}s`;
		}
		getVendorExtras(e) {
			let t = [
				"bed_temp",
				"target_bed_temp",
				"nozzle_temp",
				"target_nozzle_temp",
				"chamber_temp"
			];
			return Object.entries(e).filter(([e]) => !t.includes(e));
		}
		render() {
			return L`

      <div class="printer-dashboard">

        <div class="dashboard-header">
          <h2>Printer: ${this.printerId}</h2>
          <span class="badge badge-${this.connectionState.toLowerCase()}">
            ${this.connectionState}
          </span>
        </div>

      

        <div class="dashboard-grid">

          <div class="camera-container">
            <h3>Live Feed</h3>
            ${this.cameraUrl ? L`<img src="${this.cameraUrl}" alt="Printer Live Camera" class="printer-feed" />` : L`<div class="camera-placeholder">Camera unavailable or disabled</div>`}
          </div>

          <div class="telemetry-container">
            <div class="card">
              <h3>Job Progress</h3>
              ${this.jobStatus ? L`
                <div class="metric-row">
                  <span>State:</span> <strong>${this.jobStatus.state}</strong>
                </div>
                <div class="metric-row">
                  <span>Progress:</span> <strong>${this.jobStatus.progress}%</strong>
                </div>
                <div class="progress-bar-bg">
                  <div class="progress-bar-fill" style="width: ${this.jobStatus.progress}%"></div>
                </div>
                <div class="metric-row">
                  <span>Time Remaining:</span> <strong>${this.formatTime(this.jobStatus.time_remaining_secs)}</strong>
                </div>

                ${Object.keys(this.jobStatus.extras || {}).length > 0 ? L`
                  <details class="extras-dropdown">
                    <summary>Vendor Job Details</summary>
                    <ul>
                      ${$e(Object.entries(this.jobStatus.extras), ([e]) => e, ([e, t]) => L`
                        <li><code>${e}</code>:${JSON.stringify(t)}</li>
                      `)}
                    </ul>
                  </details>
                ` : ""}
              ` : L`<p class="muted">Waiting for job data...</p>`}
            </div>

            <div class="card">
              <h3>Temperatures</h3>
              ${this.printerStatus ? L`
                <div class="temp-grid">
                  <div class="temp-box">
                    <span class="label">Nozzle</span>
                    <span class="value">${this.printerStatus.nozzle_temp.toFixed(1)}°C</span>${this.printerStatus.target_nozzle_temp ? L`<span class="target">Target: ${this.printerStatus.target_nozzle_temp}°C</span>` : ""}
                  </div>
                  <div class="temp-box">
                    <span class="label">Bed</span>
                    <span class="value">${this.printerStatus.bed_temp.toFixed(1)}°C</span>${this.printerStatus.target_bed_temp ? L`<span class="target">Target: ${this.printerStatus.target_bed_temp}°C</span>` : ""}
                  </div>
                  ${this.printerStatus.chamber_temp !== void 0 && this.printerStatus.chamber_temp !== null ? L`
                    <div class="temp-box">
                      <span class="label">Chamber</span>
                      <span class="value">${this.printerStatus.chamber_temp.toFixed(1)}°C</span>
                    </div>
                  ` : ""}
                </div>

                ${this.getVendorExtras(this.printerStatus).length > 0 ? L`
                  <details class="extras-dropdown">
                    <summary>Hardware Extras</summary>
                    <ul>
                      ${$e(this.getVendorExtras(this.printerStatus), ([e]) => e, ([e, t]) => L`
                        <li><code>${e}</code>:${JSON.stringify(t)}</li>
                      `)}
                    </ul>
                  </details>
                ` : ""}
              ` : L`<p class="muted">Waiting for temperature telemetry...</p>`}
            </div>
          </div>
        </div>
      </div>
    `;
		}
	}, Q([q({
		type: String,
		attribute: "printer-id"
	})], $.prototype, "printerId", void 0), Q([q({
		type: String,
		attribute: "stream-url"
	})], $.prototype, "streamUrl", void 0), Q([q({
		type: String,
		attribute: "camera-url"
	})], $.prototype, "cameraUrl", void 0), Q([J()], $.prototype, "printerStatus", void 0), Q([J()], $.prototype, "jobStatus", void 0), Q([J()], $.prototype, "connectionState", void 0), Q([J()], $.prototype, "errorMessage", void 0), $ = Q([Oe("device-view")], $);
})), it = /* @__PURE__ */ t((() => {
	rt();
}));
//#endregion
export default it();
