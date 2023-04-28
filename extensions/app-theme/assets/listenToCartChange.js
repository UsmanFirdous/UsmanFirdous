//Identify cart item change
const listenToCartChange = function (ns, fetch) {
    if (typeof fetch !== "function") return;
    ns.fetch = function () {
      const response = fetch.apply(this, arguments);
      response.then((res) => {
        const isCartChanging =
          /\/cart\/add|\/cart\/update|\/cart\/change|\/cart\/clear|\/cart\/add.js|\/cart\/update.js|\/cart\/change.js|\/cart\/clear.js/.test(
            res.url.toString()
          );
        if (isCartChanging) {
          changeCartPrice();
        }
      });
      return response;
    };
};