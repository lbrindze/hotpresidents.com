
// js for hotpresidents 



var route = {};



document.addEventListener('DOMContentLoaded', function() { 
 var path = window.location.pathname.trim().split('/')[1];
 
 if(path.length>0) {
    document.querySelector(".frame").setAttribute("data-page",path); 
    route[path]();
   }
});



route.stats = function(){
    
 let hotScore = Number(document.querySelector("._score_counter._hotus").getAttribute("data-count"));
 let notScore = Number(document.querySelector("._score_counter._notus").getAttribute("data-count"));

 
                        
 var calc = Math.max(hotScore,notScore);
 let hotOrCold = hotScore == calc && notScore != calc ? "._hotus" : notScore == calc && hotScore != calc ? "._notus" : false;
    
 if(hotOrCold){
   document.querySelector("._score_counter"+hotOrCold).classList.add("_active");       
  }
       
}// end of route.score






