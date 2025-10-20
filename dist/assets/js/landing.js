"use strict";
var theme = {
  init: function () {
    theme.progressPageLoad(),
      theme.menu(),
      theme.stickyHeader()
  },
  progressPageLoad: () => {
    var e = document.querySelector(".btn-scroll-top");
    if (null != e) {
      var t = document.querySelector(".btn-scroll-top path")
        , n = t.getTotalLength();
      t.style.transition = t.style.WebkitTransition = "none",
        t.style.strokeDasharray = n + " " + n,
        t.style.strokeDashoffset = n,
        t.getBoundingClientRect(),
        t.style.transition = t.style.WebkitTransition = "stroke-dashoffset 10ms linear",
        window.addEventListener("scroll", (function (o) {
          var a = document.body.scrollTop || document.documentElement.scrollTop
            , s = document.documentElement.scrollHeight - document.documentElement.clientHeight
            , l = n - a * n / s;
          t.style.strokeDashoffset = l,
            (document.body.scrollTop || document.documentElement.scrollTop) >= 50 ? e.classList.add("active-progress") : e.classList.remove("active-progress")
        }
        )),
        e.addEventListener("click", (function (e) {
          e.preventDefault(),
            window.scroll({
              top: 0,
              left: 0,
              behavior: "smooth"
            })
        }
        ))
    }
  }
  ,
  menu: () => {
    document.querySelectorAll(".dropdown-menu a.dropdown-toggle").forEach((function (e) {
      e.addEventListener("click", (function (e) {
        if (!this.nextElementSibling.classList.contains("show")) {
          this.closest(".dropdown-menu").querySelectorAll(".show").forEach((function (e) {
            e.classList.remove("show")
          }
          ))
        }
        this.nextElementSibling.classList.toggle("show");
        const t = this.closest("li.nav-item.dropdown.show");
        t && t.addEventListener("hidden.bs.dropdown", (function (e) {
          document.querySelectorAll(".dropdown-submenu .show").forEach((function (e) {
            e.classList.remove("show")
          }
          ))
        }
        )),
          e.stopPropagation()
      }
      ))
    }
    ))
  }
  ,
  stickyHeader: () => {
    if (null != document.querySelector(".navbar"))
      new Headhesive(".navbar", {
        offset: 400,
        offsetSide: "top",
        classes: {
          clone: "navbar-clone fixed",
          stick: "navbar-stick",
          unstick: "navbar-unstick"
        },
        onStick: function () {
          var e = this.clonedElem.classList;
          e.contains("transparent") && e.contains("navbar-dark") && (this.clonedElem.className = this.clonedElem.className.replace("navbar-dark", "navbar-light", "navbar-stick"))
        }
      })
  }

};
theme.init();
var navbar = document.querySelector(".navbar");
const navOffCanvasBtn = document.querySelectorAll(".offcanvas-nav-btn")
  , navOffCanvas = document.querySelector(".navbar:not(.navbar-clone) .offcanvas-nav");
let bsOffCanvas;
function toggleOffCanvas() {
  bsOffCanvas && bsOffCanvas._isShown ? bsOffCanvas.hide() : bsOffCanvas && bsOffCanvas.show()
}
navOffCanvas && (bsOffCanvas = new bootstrap.Offcanvas(navOffCanvas, {
  scroll: !0
}),
  navOffCanvasBtn.forEach((e => {
    e.addEventListener("click", (e => {
      toggleOffCanvas()
    }
    ))
  }
  )));
