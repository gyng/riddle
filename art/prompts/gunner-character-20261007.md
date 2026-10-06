# Gunner character — built-in imagegen, 2026-10-07

Style authority: art/ART.md and docs/ART_DIRECTION.md. Existing Ranger male
sprite/portrait were inspected before editing. New assets, no replacements.
Generated portrait is opaque1254square; sprite RGBA1024x1536, transparent
corners and outside-silhouette pixels. The preview's dark/red RGB outside the
figure has alpha0. Existing pack.py consumes authored alpha, quantises/downsizes
and adds the normal ink rim; no custom background-removal or sprite editing.
Default/male aliases share one character, other looks use the existing fallback.

Output sources, generated_images/01a0fd44-e285-79f0-a67c-1d1168c32442:

- exec-00fc0a76-d785-4520-9f3f-6898d6911b19.png: initial sprite/reference.
- exec-b1ae69b7-da79-4531-84ed-ce9cf89468d1.png: final sprite, copied to
  art/generated/hero_gunner{,_male}.png.
- exec-82285f83-3451-49f6-a604-8caa819bf9ea.png: portrait, copied to
  art/ui/portraits/hero_gunner{,_male}.png; tools/ui-skin.py packs256px webp.

## Initial sprite — exact prompt

Create a new full-body Gunner class character sprite for Riddle using the attached existing Ranger character as the exact visual-style reference. Preserve the illustrated moonlit ink-and-wash gothic anime register, long roughly 1:6.5 adult figure, crisp dark silhouette outline, bone pale face patch, blue-black leather and steel with pale moonlit edges, blood-red long tattered cloak, slightly overhead three-quarter view facing right. Replace the entire bow and quiver with a clearly readable antique long gun: a long straight pale steel barrel, dark wooden stock and large flintlock mechanism, held with two hands at chest height aiming toward the right. Differentiate the Gunner with a low broad-brimmed dark hunter hat, short dark hair, fitted long coat and one broad cartridge belt; restrained gothic cartridge details, no modern tactical or science-fiction gear. Legs planted, full boots visible, cloak trails left. One single complete character, all limbs and gun entirely inside the canvas, about 10 percent transparent margins. Strong readable face and rifle silhouette at 48 pixels tall; big simple masses rather than tiny straps. No bow, arrows, sword, shield, muzzle fire, smoke, text, frame, background, environment, floor, shadow pool or glow. Actual alpha transparency. Create the sprite only, not a portrait or mockup.

Reference art/generated/hero_ranger_male.png; transparent_background=true.

## Final sprite edit — exact prompt

Edit this Gunner sprite for use as a transparent game character. Keep the same exact character, face, hunter hat, long antique rifle, ink-and-wash detail, dark blue leather, blood-red cloak, pose and proportions. Correct only the isolated asset presentation: completely remove all red/blue glow, fog, gradient, halos and any background pixels outside the actual character silhouette. The entire outer area including gaps between limbs must be fully transparent alpha, hard clear silhouette edges. Scale the complete character down slightly and center it so every tip of the cloak and the entire gun muzzle and boots have at least eight percent blank transparent margin from all four canvas edges; no cropping. No floor, no shadows, no cast shadow, no frame, no text. Preserve crisp painted interior surfaces; do not add pixel blocks or blur. One complete full-body figure facing right.

Reference initial output above; transparent_background=true. Final actual alpha
bbox27,37–994,1470; whole figure retained, margin narrower than the requested8%.
Packed crop/48px rendering inspected in real headed app; no clipped body/gun.

## Portrait — exact prompt

Create one square opaque head-and-shoulders character portrait for Riddle's Gunner. Character identity from image one: pale young adult male gothic hunter, short dark hair, low broad-brim black leather hunter hat with a small old brass buckle, long crimson cloak, dark leather coat, cartridge belt and antique long rifle. Style and portrait framing from image two: moonlit gothic anime ink-and-wash, chunky hand-inked outlines, three broad value masses with rough paper granulation, bone-ivory readable face patch, deep ink/navy backdrop, blood-red cloak only strong accent. Close-up three-quarter view facing right, character fills square, hat crown and brim remain wholly visible, eyes readable at 48px, edge of long gun held diagonally beside his shoulder to identify class without covering his face. No bow, quiver, modern weapons, sporting mascot, helmet visor, text, letter, border/frame, logo or UI. Dark subtly textured painted background. Keep elegant adult proportions and a serious attentive expression, no exaggeration.

References initial sprite and art/ui/portraits/hero_ranger_male.png;
transparent_background=false.

## Verification

495 atlas frames pass existing art QC with no warnings/failures. All493 earlier
frames retain exact dimensions and RGBA pixel content after atlas repacking.
Packed skin115icons/25portraits. Real-worker headed QA1440/400 imports a legally
unlocked earned camp after deselecting its old bow through the loadout API;
portrait idhero_gunner_male present, live map sprite inspected, no page errors
or horizontal overflow. Artifacts scratchpad/gunner-actual-kit-baseline-20261007/
art-{forge,watch}-*.png and art-qa.json. This does not certify class choice,
action icons, chamber/reload HUD or full CUT35 D.
