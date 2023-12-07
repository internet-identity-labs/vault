// bigintExtension.js
global.BigInt.prototype.toJSON = function () {
    return this.toString();
};