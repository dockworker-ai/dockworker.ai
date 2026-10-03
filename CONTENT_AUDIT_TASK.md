# Content Audit: dockworker.ai Web App

**Agent:** cursor-content  
**Scope:** Full text review of https://dockworker.ai  
**Timeline:** 2-3 hours (parallel to Phase 2)

---

## Audit Checklist

### 1. Hero Section
- [ ] **Headline:** "Build once. Promote often." — clear, compelling, accurate?
- [ ] **Subheading:** Description of apko + signed digests + immutable promotion — technically accurate?
- [ ] **CTA Buttons:** "See the pipeline" and "Compare the two paths" — clear intent?
- [ ] **Proof Points:** "apko in every path", "Nix when you want it", "Signed digest promotion" — all accurate and relevant?

### 2. Navigation & Header
- [ ] **Logo/Brand:** "Dockworker.ai" — consistent spelling, capitalization
- [ ] **Nav Links:** All present and correctly labeled
  - Home
  - Features / How it Works
  - Pricing
  - Documentation / Docs
  - GitHub / Source
- [ ] **Mobile Menu:** All nav items accessible on mobile

### 3. Pipeline / How it Works Section
- [ ] **Section Title:** Clear, matches linked anchor (#how-it-works)
- [ ] **Step Descriptions:** Git clone → BuildKit → Push → Promotion — technically accurate?
- [ ] **Supported Registries:** Docker Hub, GHCR, ECR, etc. — complete and accurate?

### 4. Comparison Section
- [ ] **Section Title:** Matches linked anchor (#comparison)
- [ ] **Column Headers:** "apko + Nix" vs "apko only" — clear distinction?
- [ ] **Row Labels:** Feature comparisons accurately describe differences
- [ ] **Checkmarks/X's:** Correct representation (not inverted logic)

### 5. Pricing Section (if present)
- [ ] **Tier Names:** Free, Pro, Team, Enterprise — consistent with backend models
- [ ] **Feature Lists:** Accurate per tier (build minutes, concurrency, cache retention)
- [ ] **CTA:** "Start free" or "Get started" — action-oriented

### 6. Features / ApkoFeature Section
- [ ] **apko Description:** Minimal, rootless, K8s-native — accurate?
- [ ] **Signed Digest Promotion:** Immutable, content-addressable — technically sound?
- [ ] **Multi-arch Support:** Mention of manifest lists, multi-platform builds — if claimed, accurate?

### 7. Call-to-Action Banners
- [ ] **CTAs:** "Start free", "Deploy now", "Build your first image" — action-oriented?
- [ ] **Secondary CTAs:** "Learn more", "View pricing" — link destinations correct?

### 8. Footer
- [ ] **Company Info:** Name, brief tagline
- [ ] **Links Present:**
  - GitHub / Repository
  - Documentation
  - Blog (if exists)
  - Privacy Policy (if exists)
  - Terms of Service (if exists)
- [ ] **Contact:** Email, support link, or contact form

### 9. General Copy Quality
- [ ] **Tone:** Professional, developer-friendly, not overly marketing-y?
- [ ] **Grammar:** No typos, consistent tense, clear sentence structure
- [ ] **Spelling:** "Kubernetes", "BuildKit", "dockworker.ai" consistently capitalized
- [ ] **Links:** All CTAs and links point to correct destinations
- [ ] **Mobile Responsiveness:** Text readable on all screen sizes (use Chrome DevTools)

### 10. Brand Consistency
- [ ] **Logo Usage:** Consistent with Figma design
- [ ] **Color Palette:** Matches design spec (blues, grays, accent colors)
- [ ] **Typography:** Font sizes, weights match design
- [ ] **Spacing:** Padding/margins consistent throughout

---

## Deliverable

**Report:** `/tmp/dockworker-content-audit.md`

```markdown
# Content Audit Results

## Summary
- Total items checked: XX
- Issues found: XX
  - Typos: N
  - Missing links: N
  - Inaccurate descriptions: N
  - Grammar issues: N

## Issues
1. [Section] [Line/Element] — Issue description + suggested fix
2. [Section] [Line/Element] — Issue description + suggested fix

## Pass/Fail by Section
| Section | Status | Notes |
|---------|--------|-------|
| Hero | ✅/❌ | |
| Nav | ✅/❌ | |
| Pipeline | ✅/❌ | |
| Comparison | ✅/❌ | |
| Features | ✅/❌ | |
| CTAs | ✅/❌ | |
| Footer | ✅/❌ | |
| Copy Quality | ✅/❌ | |

## Overall Result
- **PASS:** All sections reviewed, no critical issues
- **PASS with notes:** Minor fixes needed (typos, links)
- **FAIL:** Significant inaccuracies or missing content

## Recommended Fixes (if any)
1. [Issue] → Suggested change
2. [Issue] → Suggested change
```

---

## Execution

```bash
og-cli job submit \
  --name "content-audit" \
  --title "Content Audit: dockworker.ai Web App" \
  --description "Full text review of https://dockworker.ai for accuracy, tone, grammar, and brand consistency" \
  --agent cursor-content \
  --branch feature/content-audit \
  --labels phase2,content-review,qa
```

---

## Integration with Phase 2

This runs **in parallel** with Phase 2 (SQL, JWT, Cilium):
- **Submitted together:** All 4 Phase 2 tasks + content audit
- **Independent:** No dependencies on other tasks
- **Timeline:** 2–3 hours (completes before Phase 2 merges)
- **Result:** Content fixes applied to design branch before final merge

---

## Pass Criteria

- ✅ No typos or grammar issues
- ✅ All links/CTAs point to correct destinations
- ✅ Technical descriptions accurate (apko, BuildKit, Nix, digests)
- ✅ Copy tone matches brand voice
- ✅ Mobile responsive text
- ✅ All required sections present
