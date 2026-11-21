import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const dataDir = path.join(root, "crates", "apw-core", "data");
const sourceDir = path.join(dataDir, "us-source");

const [storesSource, proSkus, duoSkus, homeViewSkus, currentRegions] = await Promise.all([
  readFile(path.join(sourceDir, "us-apple-stores.json"), "utf8").then(JSON.parse),
  readFile(path.join(sourceDir, "iphone-18-skus.json"), "utf8").then(JSON.parse),
  readFile(path.join(sourceDir, "iphone-duo-skus.json"), "utf8").then(JSON.parse),
  readFile(path.join(sourceDir, "homeview-skus.json"), "utf8").then(JSON.parse),
  readFile(path.join(dataDir, "stores.json"), "utf8").then(JSON.parse),
]);

const stateGroups = new Map();
for (const source of storesSource) {
  if (!stateGroups.has(source.state)) stateGroups.set(source.state, []);
  const slug = new URL(source.url).pathname.split("/").filter(Boolean).at(-1) ?? "";
  stateGroups.get(source.state).push({
    id: source.storeNumber,
    name: source.storeName,
    slug,
    telephone: source.telephone ?? "",
    address: {
      address1: source.address1 ?? "",
      address2: source.address2 ?? "",
      city: source.city,
      postalCode: source.postalCode ?? "",
      __typename: "PostalAddress",
      // 核心解析器会把 stateName 放进门店标题。这里同时包含州与城市，
      // 让现有的一个搜索框可以按州、城市或店名筛选。
      stateName: `${source.state} · ${source.city}`,
      stateCode: source.stateCode ?? null,
    },
    __typename: "RgdsStore",
  });
}

const usRegion = {
  locale: "en_US",
  calledLocale: "en_US",
  state: [...stateGroups.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, store]) => ({ __typename: "RgdsState", name, store })),
  store: [],
  hasStates: true,
  __typename: "RgdsStoreSearchResult",
};

await writeFile(
  path.join(dataDir, "stores.json"),
  `${JSON.stringify([usRegion, ...currentRegions.filter((item) => item.locale !== "en_US")], null, 2)}\n`,
);

const normalized = (value) => value.toLowerCase().replaceAll(" ", "");
const models = {
  "iPhone 18 Pro": { familyType: "iphone18pro", screen: "6_3inch" },
  "iPhone 18 Pro Max": { familyType: "iphone18promax", screen: "6_9inch" },
  "iPhone Duo": { familyType: "iphoneduo", screen: "7_6inch" },
};
const seenParts = new Set();
const productSnapshot = [
  { family: "iphone-18-pro", skus: proSkus },
  { family: "iphone-duo", skus: duoSkus },
].map(({ family, skus }) => {
  const products = skus.map((sku) => {
    const model = models[sku.model];
    if (!model || !sku.partNumber.endsWith("LL/A") || seenParts.has(sku.partNumber)) {
      throw new Error(`Invalid or duplicate US SKU: ${sku.partNumber} (${sku.model})`);
    }
    seenParts.add(sku.partNumber);
    return {
      partNumber: sku.partNumber,
      familyType: model.familyType,
      dimensionCapacity: normalized(sku.storage),
      dimensionScreensize: model.screen,
      dimensionColor: normalized(sku.color),
    };
  });
  const displayValues = {
    dimensionCapacity: Object.fromEntries(
      [...new Set(skus.map((sku) => sku.storage))]
        .map((capacity) => [normalized(capacity), { value: capacity }]),
    ),
    dimensionColor: Object.fromEntries(
      [...new Set(skus.map((sku) => sku.color))]
        .map((color) => [normalized(color), { value: color }]),
    ),
    dimensionScreensize: Object.fromEntries(
      [...new Set(skus.map((sku) => sku.model))]
        .map((name) => [models[name].screen, { value: name }]),
    ),
  };
  return { category: "iphone", family, data: { products, displayValues } };
});

// HomeView 的购买页用 dimensionMerchModel 区分桌面版与壁挂版，颜色键沿用
// Apple 页面里的写法（space_gray 带下划线，rosepink 不带），这样离线快照与
// 在线刷新解析出的商品完全一致。
const homeViewModels = {
  "Apple HomeView": "homeview_standalone",
  "Apple HomeView (Wall Mounted)": "homeview_wallmount",
};
const homeViewColor = (color) => (color === "Space Gray" ? "space_gray" : normalized(color));
const homeViewProducts = homeViewSkus.map((sku) => {
  const merchModel = homeViewModels[sku.model];
  if (!merchModel || !sku.partNumber.endsWith("LL/A") || seenParts.has(sku.partNumber)) {
    throw new Error(`Invalid or duplicate US SKU: ${sku.partNumber} (${sku.model})`);
  }
  seenParts.add(sku.partNumber);
  return {
    partNumber: sku.partNumber,
    familyType: "homeview",
    dimensionMerchModel: merchModel,
    dimensionColor: homeViewColor(sku.color),
  };
});
productSnapshot.push({
  category: "home",
  family: "apple-homeview",
  data: {
    products: homeViewProducts,
    displayValues: {
      dimensionMerchModel: Object.fromEntries(
        Object.entries(homeViewModels).map(([name, key]) => [key, { value: name }]),
      ),
      dimensionColor: Object.fromEntries(
        [...new Set(homeViewSkus.map((sku) => sku.color))]
          .map((color) => [homeViewColor(color), { value: color }]),
      ),
    },
  },
});

await writeFile(
  path.join(dataDir, "products_en_US.json"),
  `${JSON.stringify(productSnapshot, null, 2)}\n`,
);

console.log(
  `Generated ${storesSource.length} US stores, ${seenParts.size - homeViewProducts.length} iPhone SKUs and ${homeViewProducts.length} HomeView SKUs.`,
);
